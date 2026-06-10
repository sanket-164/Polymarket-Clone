import type { OrderbookSnapshot } from "../types/orderbook";

interface OrderBookProps {
  snapshot: OrderbookSnapshot | undefined;
  currentPrice?: string;
}

export function OrderBook({ snapshot, currentPrice }: OrderBookProps) {
  if (!snapshot) {
    return (
      <div className="rounded-xl border border-slate-800 bg-slate-900/50 p-12 text-center text-slate-500">
        No orderbook data available for this outcome.
      </div>
    );
  }

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

  const displayPrice = currentPrice ? Number(currentPrice).toFixed(2) : "0.00";

  return (
    <section className="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
      <div className="mb-6 flex items-center justify-between">
        <h2 className="text-lg font-semibold text-white">Order Book</h2>
        <span className="rounded-md border border-slate-700 bg-slate-800 px-3 py-1 text-xs font-medium text-slate-300">
          {snapshot.outcome_label}
        </span>
      </div>

      <div className="overflow-hidden rounded-lg border border-slate-800 bg-slate-950">
        {/* Table Header */}
        <div className="grid grid-cols-2 border-b border-slate-800 bg-slate-900/50 px-4 py-2 text-xs font-medium uppercase tracking-wider text-slate-500">
          <div className="text-left">Price</div>
          <div className="text-right">Qty</div>
        </div>

        <div className="max-h-[400px] overflow-y-auto custom-scrollbar">
          {/* Asks (Sell Orders) */}
          <div className="divide-y divide-slate-800/50">
            {asks.map((ask, i) => (
              <div
                key={`ask-${i}`}
                className="relative grid grid-cols-2 px-4 py-1.5 text-sm font-mono"
              >
                <div
                  className="absolute right-0 top-0 bottom-0 bg-rose-500/10 transition-all"
                  style={{ width: `${(ask.qty / maxQty) * 100}%` }}
                />
                <div className="relative z-10 text-rose-400">
                  ${Number(ask.price).toFixed(2)}
                </div>
                <div className="relative z-10 text-right text-slate-300">
                  {ask.qty.toLocaleString()}
                </div>
              </div>
            ))}
          </div>

          {/* Current Price / Spread Indicator */}
          <div className="border-y border-slate-700 bg-slate-800/50 px-4 py-3 text-center">
            <span className="text-xs uppercase tracking-wider text-slate-500">
              Current Price
            </span>
            <div className="mt-1 text-xl font-bold text-white">
              ${displayPrice}
            </div>
          </div>

          {/* Bids (Buy Orders) */}
          <div className="divide-y divide-slate-800/50">
            {bids.map((bid, i) => (
              <div
                key={`bid-${i}`}
                className="relative grid grid-cols-2 px-4 py-1.5 text-sm font-mono"
              >
                <div
                  className="absolute right-0 top-0 bottom-0 bg-emerald-500/10 transition-all"
                  style={{ width: `${(bid.qty / maxQty) * 100}%` }}
                />
                <div className="relative z-10 text-emerald-400">
                  ${Number(bid.price).toFixed(2)}
                </div>
                <div className="relative z-10 text-right text-slate-300">
                  {bid.qty.toLocaleString()}
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}
