export type User = {
  id: string;
  name: string;
  email: string;
  picture: string | null;
  mobile_no: string | null;
  created_at: string;
  updated_at: string;
};

export type SignUpPayload = {
  name: string;
  email: string;
  password: string;
  confirmPassword: string;
};

export type SignInPayload = {
  email: string;
  password: string;
};

export type AuthTokenResponse = {
  access_token: string;
};

export type MessageResponse = {
  message: string;
};

export type ResetPasswordPayload =
  | {
    email: string;
    old_password: string;
    new_password: string;
    confirm_password: string;
  }
  | {
    email: string;
    otp: number;
    new_password: string;
    confirm_password: string;
  };

export type AccessTokenPayload = {
  sub?: string;
  exp?: number;
  iat?: number;
};
