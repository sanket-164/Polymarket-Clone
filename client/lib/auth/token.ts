import type { AccessTokenPayload } from "./types";

export function decodeAccessToken(accessToken: string): AccessTokenPayload | null {
  try {
    const [, payload] = accessToken.split(".");

    if (!payload) {
      return null;
    }

    const normalizedPayload = payload
      .replace(/-/g, "+")
      .replace(/_/g, "/")
      .padEnd(Math.ceil(payload.length / 4) * 4, "=");
    const decodedPayload = window.atob(normalizedPayload);

    return JSON.parse(decodedPayload) as AccessTokenPayload;
  } catch {
    return null;
  }
}

export function isAccessTokenExpired(accessToken: string) {
  const payload = decodeAccessToken(accessToken);

  if (!payload?.exp) {
    return false;
  }

  return payload.exp * 1000 <= Date.now();
}
