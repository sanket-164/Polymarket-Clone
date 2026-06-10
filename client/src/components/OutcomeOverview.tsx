import type { Outcome } from "../types/market-detail";

interface OutcomeOverviewProps {
  firstOutcome: Outcome;
  secondOutcome: Outcome;
  selectedOutcomeId: string | null;
  onSelect: (id: string) => void;
  currentPrices?: Record<string, string>;
}

export function OutcomeOverview({
  firstOutcome,
  secondOutcome,
  selectedOutcomeId,
  onSelect,
  currentPrices,
}: OutcomeOverviewProps) {
  return (
    <section className="grid gap-6 lg:grid-cols-2">
      <OutcomeCard
        outcome={firstOutcome}
        isSelected={selectedOutcomeId === firstOutcome.id}
        onClick={() => onSelect(firstOutcome.id)}
        liveCurrentPrice={currentPrices?.[firstOutcome.id]}
      />
      <OutcomeCard
        outcome={secondOutcome}
        isSelected={selectedOutcomeId === secondOutcome.id}
        onClick={() => onSelect(secondOutcome.id)}
        liveCurrentPrice={currentPrices?.[secondOutcome.id]}
      />
    </section>
  );
}

function OutcomeCard({
  outcome,
  isSelected,
  onClick,
  liveCurrentPrice,
}: {
  outcome: Outcome;
  isSelected: boolean;
  onClick: () => void;
  liveCurrentPrice?: string;
}) {
  const displayPrice = liveCurrentPrice
    ? `$${Number(liveCurrentPrice).toFixed(2)}`
    : `$${Number(outcome.current_price).toFixed(2)}`;

  return (
    <button
      onClick={onClick}
      className={`w-full text-left transition-all duration-200 ${
        isSelected
          ? "border-cyan-500/50 bg-slate-800/80 ring-1 ring-cyan-500/50"
          : "border-slate-800 bg-slate-900/50 hover:border-slate-700 hover:bg-slate-800/50"
      } rounded-xl border p-6`}
    >
      <div className="flex items-start justify-between gap-4">
        <div>
          <h2 className="text-lg font-semibold text-white">{outcome.label}</h2>
        </div>
        <div className="text-right">
          <p className="text-xs uppercase tracking-wider text-slate-500">
            Current Price
          </p>
          <p className="mt-1 text-xl font-bold text-emerald-400">
            {displayPrice}
          </p>
        </div>
      </div>

      <dl className="mt-6 grid gap-4 border-t border-slate-700/50 pt-4 sm:grid-cols-2">
        <Metric
          label="Start Price"
          value={`$${Number(outcome.start_price).toFixed(2)}`}
        />
        <Metric
          label="Total Shares"
          value={Number(outcome.total_shares).toLocaleString()}
        />
      </dl>
    </button>
  );
}

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <dt className="text-xs uppercase tracking-wider text-slate-500">
        {label}
      </dt>
      <dd className="mt-1 text-sm font-medium text-slate-200">{value}</dd>
    </div>
  );
}
