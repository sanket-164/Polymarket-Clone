import { AUTH_SERVICE_URL } from "./config";
import { clearStoredAccessToken, getStoredAccessToken, storeAccessToken } from "@/lib/auth/token-storage";
import type { AuthTokenResponse } from "@/lib/auth/types";

type ApiFetchOptions = RequestInit & {
  auth?: boolean;
  retryOnUnauthorized?: boolean;
};

export class ApiError extends Error {
  status: number;
  body: unknown;

  constructor(message: string, status: number, body: unknown) {
    super(message);
    this.name = "ApiError";
    this.status = status;
    this.body = body;
  }
}

export async function apiFetch<TResponse>(
  path: string,
  options: ApiFetchOptions = {},
): Promise<TResponse> {
  const { auth = true, retryOnUnauthorized = true, headers, ...requestOptions } = options;
  const response = await fetch(buildApiUrl(path), {
    ...requestOptions,
    credentials: "include",
    headers: buildHeaders(headers, auth),
  });

  if ((response.status === 401 || response.status === 403) && auth && retryOnUnauthorized) {
    const refreshedToken = await refreshAccessToken();

    if (refreshedToken) {
      return apiFetch<TResponse>(path, {
        ...options,
        retryOnUnauthorized: false,
      });
    }
  }

  return parseApiResponse<TResponse>(response);
}

async function refreshAccessToken() {
  try {
    const response = await fetch(buildApiUrl("/api/user/refresh"), {
      method: "POST",
      credentials: "include",
      headers: {
        Accept: "application/json",
      },
    });

    if (!response.ok) {
      clearStoredAccessToken();
      return null;
    }

    const data = (await response.json()) as AuthTokenResponse;
    storeAccessToken(data.access_token);

    return data.access_token;
  } catch {
    clearStoredAccessToken();
    return null;
  }
}

function buildApiUrl(path: string) {
  return `${AUTH_SERVICE_URL}${path}`;
}

function buildHeaders(headers: HeadersInit | undefined, auth: boolean) {
  const requestHeaders = new Headers(headers);

  requestHeaders.set("Accept", "application/json");

  if (!requestHeaders.has("Content-Type")) {
    requestHeaders.set("Content-Type", "application/json");
  }

  if (auth) {
    const accessToken = getStoredAccessToken();

    if (accessToken) {
      requestHeaders.set("Authorization", `Bearer ${accessToken}`);
    }
  }

  return requestHeaders;
}

async function parseApiResponse<TResponse>(response: Response) {
  const body = await parseResponseBody(response);

  if (!response.ok) {
    throw new ApiError(getErrorMessage(body, response.status), response.status, body);
  }

  return body as TResponse;
}

async function parseResponseBody(response: Response) {
  const contentType = response.headers.get("content-type");

  if (contentType?.includes("application/json")) {
    return response.json();
  }

  return response.text();
}

function getErrorMessage(body: unknown, status: number) {
  if (body && typeof body === "object" && "message" in body && typeof body.message === "string") {
    return body.message;
  }

  return `Request failed with status ${status}`;
}
