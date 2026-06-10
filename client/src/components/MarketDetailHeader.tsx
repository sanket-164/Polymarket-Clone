import { formatDateTime } from "../lib/utils";
import type { MarketDetail } from "../types/market-detail";

interface MarketDetailHeaderProps {
  market: MarketDetail;
}

export function MarketDetailHeader({ market }: MarketDetailHeaderProps) {
  return (
    <header className="rounded-xl border border-slate-800 bg-slate-900/50 p-6 sm:p-8">
      <div className="mb-6 flex flex-wrap items-center gap-2">
        <span className="rounded-md border border-slate-700 bg-slate-800 px-2.5 py-1 text-xs font-medium uppercase tracking-wider text-slate-300">
          {market.category}
        </span>
        <span className="rounded-md border border-slate-700 bg-slate-800 px-2.5 py-1 text-xs font-medium uppercase tracking-wider text-slate-300">
          {market.status}
        </span>
      </div>

      <h1 className="text-2xl font-bold tracking-tight text-white sm:text-3xl">
        {market.title}
      </h1>

      <p className="mt-4 max-w-3xl text-sm leading-relaxed text-slate-400 sm:text-base">
        {market.description}
      </p>

      <div className="mt-8 grid gap-6 border-t border-slate-800 pt-6 sm:grid-cols-3">
        <InfoItem label="Start date" value={formatDateTime(market.start_at)} />
        <InfoItem label="Close date" value={formatDateTime(market.close_at)} />
        {market.deleted_at && (
          <InfoItem label="Deleted" value={formatDateTime(market.deleted_at)} />
        )}
      </div>
    </header>
  );
}

function InfoItem({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <p className="text-xs uppercase tracking-wider text-slate-500">{label}</p>
      <p className="mt-1 text-sm font-medium text-slate-200">{value}</p>
    </div>
  );
}
