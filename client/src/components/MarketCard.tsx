import { Link } from "react-router-dom";
import type { Market } from "../types/market";
import { formatDateTime, getStatusColor } from "../lib/utils";

interface MarketCardProps {
  market: Market;
}

export function MarketCard({ market }: MarketCardProps) {
  return (
    <Link
      to={`/markets/${market.id}`}
      className="group block rounded-xl border border-slate-800 bg-slate-900/50 p-6 transition-all hover:border-cyan-500/50 hover:bg-slate-800/50 hover:shadow-lg hover:shadow-cyan-500/10"
    >
      <div className="flex items-start justify-between gap-4">
        <div className="flex-1">
          <div className="mb-3 flex items-center gap-2">
            <span className="rounded-full border border-slate-700 bg-slate-800 px-2.5 py-0.5 text-xs font-medium text-slate-300">
              {market.category}
            </span>
            <span
              className={`rounded-full border px-2.5 py-0.5 text-xs font-medium uppercase tracking-wide ${getStatusColor(
                market.status
              )}`}
            >
              {market.status}
            </span>
          </div>

          <h3 className="mb-2 text-lg font-semibold text-slate-100 transition-colors group-hover:text-cyan-300">
            {market.title}
          </h3>

          <p className="mb-4 line-clamp-2 text-sm text-slate-400">
            {market.description}
          </p>
        </div>
      </div>

      <div className="mt-4 flex items-center gap-6 border-t border-slate-800 pt-4 text-xs text-slate-500">
        <div>
          <span className="block font-medium text-slate-400">Starts</span>
          {formatDateTime(market.start_at)}
        </div>
        <div>
          <span className="block font-medium text-slate-400">Closes</span>
          {formatDateTime(market.close_at)}
        </div>
      </div>
    </Link>
  );
}
