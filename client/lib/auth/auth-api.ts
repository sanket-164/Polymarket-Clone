import { apiFetch } from "@/lib/api/http";
import { clearStoredAccessToken, storeAccessToken } from "./token-storage";
import type {
  AuthTokenResponse,
  MessageResponse,
  ResetPasswordPayload,
  SignInPayload,
  SignUpPayload,
  User,
} from "./types";

export async function signUp(payload: SignUpPayload) {
  return apiFetch<User>("/api/auth/user/signup", {
    method: "POST",
    auth: false,
    body: JSON.stringify(payload),
  });
}

export async function signIn(payload: SignInPayload) {
  const response = await apiFetch<AuthTokenResponse>("/api/auth/user/signin", {
    method: "POST",
    auth: false,
    body: JSON.stringify(payload),
  });

  storeAccessToken(response.access_token);

  return response;
}

export async function sendOtp(email: string) {
  return apiFetch<MessageResponse>("/api/auth/user/send-otp", {
    method: "POST",
    auth: false,
    body: JSON.stringify({ email }),
  });
}

export async function resetPassword(payload: ResetPasswordPayload) {
  return apiFetch<MessageResponse>("/api/auth/user/reset-password", {
    method: "POST",
    auth: false,
    body: JSON.stringify(payload),
  });
}

export async function refreshSession() {
  return apiFetch<AuthTokenResponse>("/api/auth/user/refresh", {
    method: "POST",
    auth: false,
  });
}

export async function logout() {
  try {
    return await apiFetch<MessageResponse>("/api/auth/user/logout", {
      method: "POST",
      auth: false,
    });
  } finally {
    clearStoredAccessToken();
  }
}
