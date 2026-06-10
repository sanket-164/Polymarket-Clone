import { useEffect, useState } from "react";
import { MarketCard } from "../components/MarketCard";
import { fetchMarkets } from "../api/market";
import type { Market } from "../types/market";

export function MarketsPage() {
  const [markets, setMarkets] = useState<Market[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const controller = new AbortController();

    async function loadMarkets() {
      try {
        setLoading(true);
        const data = await fetchMarkets(controller.signal);
        setMarkets(data);
      } catch (error) {
        if ((error as Error).name !== "AbortError") {
          console.error("Error fetching markets:", error);
        }
      } finally {
        setLoading(false);
      }
    }

    loadMarkets();

    return () => controller.abort();
  }, []);

  return (
    <main className="min-h-screen bg-slate-950 text-slate-100">
      <div className="mx-auto max-w-7xl px-4 py-12 sm:px-6 lg:px-8">
        {loading ? (
          <div className="grid gap-6 md:grid-cols-2 xl:grid-cols-3">
            {Array.from({ length: 6 }).map((_, index) => (
              <div
                key={index}
                className="h-96 animate-pulse rounded-xl border border-slate-800 bg-slate-900"
              />
            ))}
          </div>
        ) : markets.length === 0 ? (
          <div className="rounded-xl border border-slate-800 bg-slate-900 p-12 text-center text-slate-400">
            No markets found.
          </div>
        ) : (
          <div className="grid gap-6 md:grid-cols-2 xl:grid-cols-3">
            {markets.map((market) => (
              <MarketCard key={market.id} market={market} />
            ))}
          </div>
        )}
      </div>
    </main>
  );
}
