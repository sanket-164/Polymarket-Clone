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
  currentPrices?: Record<string, string>;
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
        </div>

        {/* Outcome Toggle Tabs */}
        <div className="flex gap-1 rounded-lg border border-border bg-card p-1">
          {snapshot.snapshot.map((outcome) => (
            <button
              key={outcome.outcome_id}
              onClick={() => setSelectedOutcomeId(outcome.outcome_id)}
              className={`rounded-md px-4 py-1.5 text-sm font-medium transition w-full ${
                selectedOutcomeId === outcome.outcome_id
                  ? "text-text shadow-sm bg-blue-600"
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
  // Sort asks ascending (lowest price first = closest to spread), take top 5,
  // then reverse so the highest of those 5 is at the top and lowest is at the bottom.
  const asks = [...snapshot.sell]
    .sort((a, b) => Number(a.price) - Number(b.price))
    .slice(0, 5)
    .reverse();

  // Sort bids descending (highest price first = closest to spread), take top 5.
  const bids = [...snapshot.buy]
    .sort((a, b) => Number(b.price) - Number(a.price))
    .slice(0, 5);

  const maxQty = Math.max(
    1,
    ...asks.map((a) => a.qty),
    ...bids.map((b) => b.qty)
  );

  const displayPrice = currentPrice ?? "--";

  // Pad asks at the BEGINNING so actual asks sit at the bottom (closest to spread/current price)
  const displayAsks: ((typeof asks)[0] | null)[] = [
    ...Array(5 - asks.length).fill(null),
    ...asks,
  ];

  // Pad bids at the END so actual bids sit at the top (closest to spread/current price)
  const displayBids: ((typeof bids)[0] | null)[] = [
    ...bids,
    ...Array(5 - bids.length).fill(null),
  ];

  return (
    <div className="overflow-hidden rounded-xl border border-border bg-card">
      {/* Table Header */}
      <div className="grid grid-cols-2 border-b border-border bg-surface px-4 py-2 text-xs font-medium uppercase tracking-wider text-secondary">
        <div className="text-left">Price</div>
        <div className="text-right">Qty</div>
      </div>

      {/* Asks (Sell Orders) */}
      <div className="divide-y divide-border/50">
        {displayAsks.map((ask, i) => (
          <div
            key={ask ? `ask-${ask.price}-${ask.qty}-${i}` : `ask-empty-${i}`}
            className="relative grid grid-cols-2 px-4 py-2 text-sm font-mono transition-colors hover:bg-surface/50"
          >
            {ask ? (
              <>
                <div
                  className="absolute right-0 top-0 bottom-0 bg-sell/10 transition-all"
                  style={{ width: `${(ask.qty / maxQty) * 100}%` }}
                />
                <div className="relative z-10 text-sell">
                  {Number(ask.price).toFixed(2)}
                </div>
                <div className="relative z-10 text-right text-text">
                  {ask.qty}
                </div>
              </>
            ) : (
              <>
                <div className="relative z-10 text-sell/20">--</div>
                <div className="relative z-10 text-right text-text/20">--</div>
              </>
            )}
          </div>
        ))}
      </div>

      {/* Current Price / Spread Indicator */}
      <div className="border-y border-border bg-surface/50 px-4 py-3 text-center">
        <div className="mt-1 text-xl font-bold text-text">
          {displayPrice === "--" ? "--" : Number(displayPrice).toFixed(2)}
        </div>
      </div>

      {/* Bids (Buy Orders) */}
      <div className="divide-y divide-border/50">
        {displayBids.map((bid, i) => (
          <div
            key={bid ? `bid-${bid.price}-${bid.qty}-${i}` : `bid-empty-${i}`}
            className="relative grid grid-cols-2 px-4 py-2 text-sm font-mono transition-colors hover:bg-surface/50"
          >
            {bid ? (
              <>
                <div
                  className="absolute right-0 top-0 bottom-0 bg-buy/10 transition-all"
                  style={{ width: `${(bid.qty / maxQty) * 100}%` }}
                />
                <div className="relative z-10 text-buy">
                  {Number(bid.price).toFixed(2)}
                </div>
                <div className="relative z-10 text-right text-text">
                  {bid.qty}
                </div>
              </>
            ) : (
              <>
                <div className="relative z-10 text-buy/20">--</div>
                <div className="relative z-10 text-right text-text/20">--</div>
              </>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
