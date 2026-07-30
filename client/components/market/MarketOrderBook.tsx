"use client";

import { useState } from "react";
import type {
  MarketDetails,
  MarketSnapshot,
  OutcomeSnapshot,
} from "@/lib/market/types";

type MarketOrderBookProps = {
  snapshot: MarketSnapshot;
  firstOutcome: MarketDetails["first_outcome"];
  secondOutcome: MarketDetails["second_outcome"];
  currentPrices?: Record<string, string>; // Added real-time prices prop
};

export function MarketOrderBook({
  snapshot,
  firstOutcome,
  secondOutcome,
  currentPrices = {},
}: MarketOrderBookProps) {
  const [selectedOutcomeId, setSelectedOutcomeId] = useState<string>(
    snapshot.snapshot[0]?.outcome_id || ""
  );

  const selectedOutcome = snapshot.snapshot.find(
    (o) => o.outcome_id === selectedOutcomeId
  );

  // Use the real-time price from WebSocket if available, otherwise fallback to initial static price
  const realTimePrice = currentPrices[selectedOutcomeId];
  const staticPrice =
    selectedOutcomeId === firstOutcome.id
      ? firstOutcome.current_price
      : secondOutcome.current_price;

  const currentPrice = realTimePrice ?? staticPrice;

  if (!selectedOutcome) return null;

  return (
    <div className="rounded-2xl border border-border bg-surface p-6">
      <div className="mb-6 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h2 className="text-lg font-semibold text-text">Order Book</h2>
          <p className="text-sm text-secondary">Live market depth</p>
        </div>

        {/* Outcome Toggle Tabs */}
        <div className="flex gap-1 rounded-lg border border-border bg-card p-1">
          {snapshot.snapshot.map((outcome) => (
            <button
              key={outcome.outcome_id}
              onClick={() => setSelectedOutcomeId(outcome.outcome_id)}
              className={`rounded-md px-4 py-1.5 text-sm font-medium transition ${
                selectedOutcomeId === outcome.outcome_id
                  ? "bg-surface text-text shadow-sm"
                  : "text-secondary hover:text-text"
              }`}
            >
              {outcome.outcome_label}
            </button>
          ))}
        </div>
      </div>

      <OrderBookView snapshot={selectedOutcome} currentPrice={currentPrice} />
    </div>
  );
}

function OrderBookView({
  snapshot,
  currentPrice,
}: {
  snapshot: OutcomeSnapshot;
  currentPrice: string;
}) {
  // Sort asks (sell) descending: highest price at top, lowest at bottom (closest to spread)
  const asks = [...snapshot.sell].sort(
    (a, b) => Number(b.price) - Number(a.price)
  );

  // Sort bids (buy) descending: highest price at top (closest to spread), lowest at bottom
  const bids = [...snapshot.buy].sort(
    (a, b) => Number(b.price) - Number(a.price)
  );

  const maxQty = Math.max(
    1,
    ...asks.map((a) => a.qty),
    ...bids.map((b) => b.qty)
  );

  const displayPrice = currentPrice ?? "--";

  if (asks.length === 0 && bids.length === 0) {
    return (
      <div className="rounded-xl border border-border bg-card p-12 text-center text-secondary">
        No orderbook data available for this outcome.
      </div>
    );
  }

  return (
    <div className="overflow-hidden rounded-xl border border-border bg-card">
      {/* Table Header */}
      <div className="grid grid-cols-2 border-b border-border bg-surface px-4 py-2 text-xs font-medium uppercase tracking-wider text-secondary">
        <div className="text-left">Price</div>
        <div className="text-right">Qty</div>
      </div>

      <div className="max-h-[400px] overflow-y-auto">
        {/* Asks (Sell Orders) */}
        <div className="divide-y divide-border/50">
          {asks.map((ask, i) => (
            <div
              key={`ask-${i}`}
              className="relative grid grid-cols-2 px-4 py-2 text-sm font-mono transition-colors hover:bg-surface/50"
            >
              <div
                className="absolute right-0 top-0 bottom-0 bg-sell/10 transition-all"
                style={{ width: `${(ask.qty / maxQty) * 100}%` }}
              />
              <div className="relative z-10 text-sell">{ask.price}</div>
              <div className="relative z-10 text-right text-text">
                {ask.qty}
              </div>
            </div>
          ))}
        </div>

        {/* Current Price / Spread Indicator */}
        <div className="border-y border-border bg-surface/50 px-4 py-3 text-center">
          <span className="text-xs uppercase tracking-wider text-secondary">
            Current Price
          </span>
          <div className="mt-1 text-xl font-bold text-text">
            {Number(displayPrice)}
          </div>
        </div>

        {/* Bids (Buy Orders) */}
        <div className="divide-y divide-border/50">
          {bids.map((bid, i) => (
            <div
              key={`bid-${i}`}
              className="relative grid grid-cols-2 px-4 py-2 text-sm font-mono transition-colors hover:bg-surface/50"
            >
              <div
                className="absolute right-0 top-0 bottom-0 bg-buy/10 transition-all"
                style={{ width: `${(bid.qty / maxQty) * 100}%` }}
              />
              <div className="relative z-10 text-buy">{bid.price}</div>
              <div className="relative z-10 text-right text-text">
                {bid.qty}
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
