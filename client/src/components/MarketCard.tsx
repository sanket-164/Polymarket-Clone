import { Link } from "react-router-dom";
import type { Market } from "../types/market";
import { formatDateTime, getStatusColor } from "../lib/utils";

interface MarketCardProps {
  market: Market;
}

export function MarketCard({ market }: MarketCardProps) {
  return (
    <article className="flex flex-col rounded-xl border border-slate-800 bg-slate-900/50 p-6 transition-colors duration-200 hover:border-slate-700 hover:bg-slate-900">
      <div className="mb-4 flex flex-wrap items-center gap-2">
        <span className="rounded-md border border-slate-700 bg-slate-800 px-2.5 py-1 text-xs font-medium uppercase tracking-wider text-slate-300">
          {market.category}
        </span>
        <span
          className={`rounded-md px-2.5 py-1 text-xs font-medium uppercase tracking-wider ${getStatusColor(market.status)}`}
        >
          {market.status}
        </span>
      </div>

      <h2 className="mb-3 text-lg font-semibold leading-snug text-white">
        {market.title}
      </h2>

      <p className="mb-6 line-clamp-3 text-sm leading-relaxed text-slate-400">
        {market.description}
      </p>

      <div className="mt-auto space-y-3 border-t border-slate-800 pt-4 text-sm">
        <div className="grid grid-cols-2 gap-x-4 gap-y-3">
          <div>
            <span className="block text-xs uppercase tracking-wider text-slate-500">
              Starts
            </span>
            <span className="text-slate-200">
              {formatDateTime(market.start_at)}
            </span>
          </div>
          <div>
            <span className="block text-xs uppercase tracking-wider text-slate-500">
              Closes
            </span>
            <span className="text-slate-200">
              {formatDateTime(market.close_at)}
            </span>
          </div>
        </div>
      </div>

      <div className="mt-6">
        <Link
          to={`/markets/${market.id}`}
          className="flex w-full items-center justify-center rounded-lg bg-slate-800 px-4 py-2.5 text-sm font-medium text-white transition-colors hover:bg-slate-700"
        >
          View Details
        </Link>
      </div>
    </article>
  );
}
