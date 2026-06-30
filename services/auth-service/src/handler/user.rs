use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
    routing::post,
};
use axum_extra::extract::cookie::Cookie;
use common::{
    constant::{OTP_CACHE_TTL, RESET_PASSWORD, SEND_OTP, SIGNIN, SIGNUP},
    error::{ErrorMessage, HttpError},
    util::{hash, jwt},
};
use deadpool_redis::redis::AsyncCommands;
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor, message::header::ContentType,
    transport::smtp::authentication::Credentials,
};
use rand::Rng;
use serde_json::json;
use validator::Validate;

use crate::{
    AppState,
    db::AuthExt,
    dto::{LoginUserDTO, RegisterUserDTO, ResetPassword, SendOtpDTO, UserResponse},
};

pub fn user_auth_handler() -> Router<Arc<AppState>> {
    Router::new()
        .route(SIGNUP, post(signup))
        .route(SIGNIN, post(signin))
        .route(SEND_OTP, post(send_otp))
        .route(RESET_PASSWORD, post(reset_password))
}

fn build_otp_email_html(name: &str, otp: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
            <html>
            <head>
            <meta charset="utf-8" />
            <meta name="viewport" content="width=device-width, initial-scale=1.0" />
            <title>Your Verification Code</title>
            </head>
            <body style="margin:0; padding:0; background-color:#f4f5f7; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;">
            <table role="presentation" width="100%" cellpadding="0" cellspacing="0" style="background-color:#f4f5f7; padding: 40px 0;">
                <tr>
                <td align="center">
                    <table role="presentation" width="480" cellpadding="0" cellspacing="0" style="background-color:#ffffff; border-radius:12px; overflow:hidden; box-shadow: 0 2px 8px rgba(0,0,0,0.06);">
                    <tr>
                        <td style="padding: 40px 40px 24px 40px; text-align:center;">
                        <h1 style="margin:0; font-size:20px; color:#111827; font-weight:600;">Verify your identity</h1>
                        </td>
                    </tr>
                    <tr>
                        <td style="padding: 0 40px 8px 40px; text-align:center;">
                        <p style="margin:0; font-size:15px; color:#4b5563; line-height:1.5;">
                            Hi {name}, use the code below to complete your verification. This code will expire in 5 minutes.
                        </p>
                        </td>
                    </tr>
                    <tr>
                        <td style="padding: 32px 40px; text-align:center;">
                        <div style="display:inline-block; background-color:#f3f4f6; border-radius:8px; padding:16px 32px;">
                            <span style="font-size:32px; font-weight:700; letter-spacing:8px; color:#111827;">{otp}</span>
                        </div>
                        </td>
                    </tr>
                    <tr>
                        <td style="padding: 0 40px 32px 40px; text-align:center;">
                        <p style="margin:0; font-size:13px; color:#9ca3af; line-height:1.5;">
                            If you didn't request this code, you can safely ignore this email.
                        </p>
                        </td>
                    </tr>
                    <tr>
                        <td style="padding: 20px 40px; border-top:1px solid #f0f0f0; text-align:center;">
                        <p style="margin:0; font-size:12px; color:#9ca3af;">&copy; {year} Your Company. All rights reserved.</p>
                        </td>
                    </tr>
                    </table>
                </td>
                </tr>
            </table>
            </body>
            </html>"#,
        name = name,
        otp = otp,
        year = chrono::Utc::now().format("%Y")
    )
}

pub async fn signup(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<RegisterUserDTO>,
) -> Result<impl IntoResponse, HttpError> {
    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let existing_user = app_state
        .pg_client
        .get_user_by_email(&body.email)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    if existing_user.is_some() {
        Err(HttpError::conflict(ErrorMessage::EmailExist.to_string()))?
    }

    let hash_password =
        hash::generate(body.password).map_err(|e| HttpError::server_error(e.to_string()))?;

    let new_user = app_state
        .pg_client
        .create_user(&body.name, &body.email, &hash_password)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok((StatusCode::CREATED, Json(UserResponse::from(new_user))))
}

pub async fn signin(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<LoginUserDTO>,
) -> Result<impl IntoResponse, HttpError> {
    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let existing_user = app_state
        .pg_client
        .get_user_by_email(&body.email)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let user = existing_user.ok_or(HttpError::unauthorized(
        ErrorMessage::WrongCredentials.to_string(),
    ))?;

    let password_matched = hash::compare(&body.password, &user.password)
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    if password_matched {
        let jwt_token = jwt::generate_token(
            &user.id.to_string(),
            app_state.jwt_config.jwt_secret_key.as_bytes(),
            app_state.jwt_config.jwt_expiration_time,
        )
        .map_err(|e| HttpError::server_error(e.to_string()))?;

        let cookie_duration =
            time::Duration::minutes(app_state.jwt_config.jwt_expiration_time as i64 * 60);

        let cookie = Cookie::build(("token", jwt_token.clone()))
            .path("/")
            .max_age(cookie_duration)
            .http_only(true)
            .build();

        let response = (
            StatusCode::OK,
            Json(json!({
                "token": jwt_token
            })),
        );

        let mut headers = HeaderMap::new();

        headers.append(header::SET_COOKIE, cookie.to_string().parse().unwrap());

        let mut response = response.into_response();
        response.headers_mut().extend(headers);

        Ok(response)
    } else {
        Err(HttpError::unauthorized(
            ErrorMessage::WrongCredentials.to_string(),
        ))
    }
}

pub async fn send_otp(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<SendOtpDTO>,
) -> Result<impl IntoResponse, HttpError> {
    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let existing_user = app_state
        .pg_client
        .get_user_by_email(&body.email)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let user = existing_user.ok_or(HttpError::unauthorized(
        ErrorMessage::WrongCredentials.to_string(),
    ))?;

    let mut redis = app_state
        .redis_pool
        .get()
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let otp: u32 = rand::thread_rng().gen_range(100_000..=999_999);
    let otp_str = otp.to_string();

    let redis_key = format!("otp:{}", body.email);
    redis
        .set_ex::<_, _, ()>(&redis_key, &otp_str, OTP_CACHE_TTL)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let html_body = build_otp_email_html(&user.name, &otp_str);

    let email = Message::builder()
        .from(
            app_state.smtp_config.smtp_username.parse().map_err(
                |e: lettre::address::AddressError| HttpError::server_error(e.to_string()),
            )?,
        )
        .to(body
            .email
            .parse()
            .map_err(|e: lettre::address::AddressError| HttpError::server_error(e.to_string()))?)
        .subject("Your Verification Code")
        .header(ContentType::TEXT_HTML)
        .body(html_body)
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let creds = Credentials::new(
        app_state.smtp_config.smtp_username.clone(),
        app_state.smtp_config.smtp_password.clone(),
    );

    let mailer: AsyncSmtpTransport<Tokio1Executor> =
        AsyncSmtpTransport::<Tokio1Executor>::relay(&app_state.smtp_config.smtp_host)
            .map_err(|e| HttpError::server_error(e.to_string()))?
            .credentials(creds)
            .build();

    mailer
        .send(email)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "message": "OTP sent to your email"
    })))
}

pub async fn reset_password(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<ResetPassword>,
) -> Result<impl IntoResponse, HttpError> {
    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    if body.old_password.is_none() && body.otp.is_none() {
        return Err(HttpError::bad_request(
            ErrorMessage::OtpOrPasswordRequired.to_string(),
        ));
    }

    let existing_user = app_state
        .pg_client
        .get_user_by_email(&body.email)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let user = existing_user.ok_or(HttpError::unauthorized(
        ErrorMessage::WrongCredentials.to_string(),
    ))?;

    if let Some(old_password) = body.old_password {
        let password_matched = hash::compare(&old_password, &user.password)
            .map_err(|e| HttpError::server_error(e.to_string()))?;

        if !password_matched {
            return Err(HttpError::unauthorized(
                ErrorMessage::WrongCredentials.to_string(),
            ));
        }
    } else if let Some(otp) = body.otp {
        let mut redis = app_state
            .redis_pool
            .get()
            .await
            .map_err(|e| HttpError::server_error(e.to_string()))?;

        let redis_key = format!("otp:{}", body.email);

        let stored_otp: Option<String> = redis
            .get(&redis_key)
            .await
            .map_err(|e| HttpError::server_error(e.to_string()))?;

        let stored_otp = stored_otp.ok_or(HttpError::unauthorized(
            "OTP has expired or is invalid".to_string(),
        ))?;

        if stored_otp != otp.to_string() {
            return Err(HttpError::unauthorized("Invalid OTP".to_string()));
        }

        redis
            .del::<_, ()>(&redis_key)
            .await
            .map_err(|e| HttpError::server_error(e.to_string()))?;
    }

    let hashed_password =
        hash::generate(body.new_password).map_err(|e| HttpError::server_error(e.to_string()))?;

    app_state
        .pg_client
        .reset_password(&body.email, hashed_password)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "message": "Password reset successfully"
    })))
}
