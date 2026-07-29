import { MARKET_SERVICE_URL } from "@/lib/api/config";
import { apiFetch } from "@/lib/api/http";
import type { Market, MarketDetails, MarketSnapshot, MarketsQuery } from "@/lib/market/types";

export async function getMarkets(query: MarketsQuery = {}) {
  const searchParams = new URLSearchParams();

  Object.entries(query).forEach(([key, value]) => {
    if (value !== undefined && value !== "") {
      searchParams.set(key, String(value));
    }
  });

  const queryString = searchParams.toString();

  return apiFetch<Market[]>(`/api/market${queryString ? `?${queryString}` : ""}`, {
    baseUrl: MARKET_SERVICE_URL,
  });
}

export async function getMarketDetails(marketId: string) {
  return apiFetch<MarketDetails>(`/api/market/${marketId}`, {
    baseUrl: MARKET_SERVICE_URL,
  });
}

export async function getMarketSnapshot(marketId: string) {
  return apiFetch<MarketSnapshot>(`/api/market/snapshot/${marketId}`, {
    baseUrl: MARKET_SERVICE_URL,
  });
}
