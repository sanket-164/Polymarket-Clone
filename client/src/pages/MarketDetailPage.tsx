import { useEffect, useState } from "react";
import { Link, Navigate, useParams } from "react-router-dom";
import { fetchMarketById, fetchOrderbookSnapshot } from "../api/market";
import { MarketDetailHeader } from "../components/MarketDetailHeader";
import { OutcomeOverview } from "../components/OutcomeOverview";
import { OrderBook } from "../components/OrderBook";
import { useOrderbookWebSocket } from "../hooks/useOrderbookWebSocket";
import type { MarketDetail } from "../types/market-detail";
import type { OrderbookSnapshot } from "../types/orderbook";

export function MarketDetailPage() {
  const { marketId } = useParams();
  const [market, setMarket] = useState<MarketDetail | null>(null);
  const [initialSnapshots, setInitialSnapshots] = useState<OrderbookSnapshot[]>(
    []
  );
  const [selectedOutcomeId, setSelectedOutcomeId] = useState<string | null>(
    null
  );
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Connect WebSocket only after initial REST data is loaded
  const { snapshots, currentPrices } = useOrderbookWebSocket(
    marketId || "",
    initialSnapshots,
    !loading && initialSnapshots.length > 0
  );

  useEffect(() => {
    if (!marketId) return;

    const controller = new AbortController();

    async function loadMarket() {
      try {
        setLoading(true);
        setError(null);

        const [marketResponse, snapshotResponse] = await Promise.all([
          fetchMarketById(marketId as string, controller.signal),
          fetchOrderbookSnapshot(marketId as string, controller.signal),
        ]);

        setMarket(marketResponse);
        setInitialSnapshots(snapshotResponse);

        if (marketResponse?.first_outcome?.id) {
          setSelectedOutcomeId(marketResponse.first_outcome.id);
        }
      } catch (err) {
        if (
          (err as Error).name !== "CanceledError" &&
          (err as Error).name !== "AbortError"
        ) {
          setError(
            err instanceof Error ? err.message : "Failed to load market"
          );
        }
      } finally {
        setLoading(false);
      }
    }

    loadMarket();
    return () => controller.abort();
  }, [marketId]);

  if (!marketId) {
    return <Navigate to="/markets" replace />;
  }

  const selectedOutcome = market
    ? market.first_outcome.id === selectedOutcomeId
      ? market.first_outcome
      : market.second_outcome
    : null;

  const selectedSnapshot = snapshots.find(
    (s) => s.outcome_id === selectedOutcomeId
  );

  // Use live price if available, otherwise fallback to REST API price
  const liveCurrentPrice =
    selectedOutcomeId && currentPrices[selectedOutcomeId]
      ? currentPrices[selectedOutcomeId]
      : selectedOutcome?.current_price;

  return (
    <main className="min-h-screen bg-slate-950 text-slate-100">
      <div className="mx-auto max-w-7xl px-4 py-12 sm:px-6 lg:px-8">
        <div className="mb-8">
          <Link
            to="/markets"
            className="inline-flex items-center rounded-lg border border-slate-800 bg-slate-900 px-4 py-2 text-sm font-medium text-slate-300 transition-colors hover:border-slate-700 hover:bg-slate-800 hover:text-white"
          >
            Back to markets
          </Link>
        </div>

        {loading ? (
          <div className="space-y-6">
            <div className="h-64 animate-pulse rounded-xl border border-slate-800 bg-slate-900" />
            <div className="grid gap-6 lg:grid-cols-2">
              <div className="h-48 animate-pulse rounded-xl border border-slate-800 bg-slate-900" />
              <div className="h-48 animate-pulse rounded-xl border border-slate-800 bg-slate-900" />
            </div>
            <div className="h-96 animate-pulse rounded-xl border border-slate-800 bg-slate-900" />
          </div>
        ) : error ? (
          <div className="rounded-xl border border-red-500/20 bg-red-500/10 p-6 text-red-200">
            <p className="text-lg font-semibold">Unable to load this market.</p>
            <p className="mt-2 text-sm text-red-200/80">{error}</p>
          </div>
        ) : !market ? (
          <div className="rounded-xl border border-slate-800 bg-slate-900 p-12 text-center text-slate-400">
            No market matched this ID.
          </div>
        ) : (
          <div className="space-y-8 pb-10">
            <MarketDetailHeader market={market} />
            <OutcomeOverview
              firstOutcome={market.first_outcome}
              secondOutcome={market.second_outcome}
              selectedOutcomeId={selectedOutcomeId}
              onSelect={setSelectedOutcomeId}
              currentPrices={currentPrices}
            />
            <OrderBook
              snapshot={selectedSnapshot}
              currentPrice={liveCurrentPrice}
            />
          </div>
        )}
      </div>
    </main>
  );
}
