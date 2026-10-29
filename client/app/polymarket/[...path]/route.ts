import type { NextRequest } from "next/server";

const BACKEND_API_URL = process.env.BACKEND_API_URL;
const HOP_BY_HOP_HEADERS = new Set([
  "connection",
  "content-length",
  "host",
  "keep-alive",
  "proxy-authenticate",
  "proxy-authorization",
  "te",
  "trailer",
  "transfer-encoding",
  "upgrade",
]);

type ProxyContext = {
  params: Promise<{ path: string[] }>;
};

async function proxyRequest(request: NextRequest, { params }: ProxyContext) {
  if (!BACKEND_API_URL) {
    return Response.json({ message: "Backend API is not configured" }, { status: 500 });
  }

  const { path } = await params;
  const backendUrl = new URL(
    `/${path.map((segment) => encodeURIComponent(segment)).join("/")}${request.nextUrl.search}`,
    `${BACKEND_API_URL.replace(/\/+$/, "")}/`,
  );
  const headers = forwardHeaders(request.headers);
  const hasBody = request.method !== "GET" && request.method !== "HEAD";
  const body = hasBody ? await request.arrayBuffer() : undefined;

  try {
    const response = await fetch(backendUrl, {
      method: request.method,
      headers,
      body,
      cache: "no-store",
    });

    return new Response(response.body, {
      status: response.status,
      statusText: response.statusText,
      headers: forwardHeaders(response.headers),
    });
  } catch (error) {
    console.error("Backend proxy request failed", {
      url: backendUrl.toString(),
      error,
    });

    return Response.json({ message: "Unable to reach the backend API" }, { status: 502 });
  }
}

function forwardHeaders(headers: Headers) {
  const forwardedHeaders = new Headers();

  headers.forEach((value, key) => {
    if (!HOP_BY_HOP_HEADERS.has(key.toLowerCase())) {
      forwardedHeaders.set(key, value);
    }
  });

  return forwardedHeaders;
}

export const GET = proxyRequest;
export const HEAD = proxyRequest;
export const POST = proxyRequest;
export const PUT = proxyRequest;
export const PATCH = proxyRequest;
export const DELETE = proxyRequest;
