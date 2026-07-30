"use client";

import Link from "next/link";
import { useEffect, useMemo, useState } from "react";
import { useAuth } from "@/components/auth/AuthProvider";
import {
  formatDateTime,
  getStatusClassName,
} from "@/components/market/market-format";
import { ApiError } from "@/lib/api/http";
import { getMarkets } from "@/lib/market/market-api";
import type { Market, MarketStatus } from "@/lib/market/types";

const MARKET_STATUSES: Array<MarketStatus> = [
  "ACTIVE",
  "PENDING",
  "CLOSED",
  "RESOLVED",
  "CANCELLED",
];

export function MarketsHome() {
  const { isLoading } = useAuth();
  const [markets, setMarkets] = useState<Market[]>([]);
  const [status, setStatus] = useState<MarketStatus>("ACTIVE");
  const [category, setCategory] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [isMarketsLoading, setIsMarketsLoading] = useState(true);

  useEffect(() => {
    if (isLoading) {
      return;
    }

    let isCurrent = true;
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setIsMarketsLoading(true);
    setError(null);

    getMarkets({
      order_field: "created_at",
      order_by: "DESC",
      status,
      category: category.trim(),
    })
      .then((marketResponse) => {
        if (isCurrent) {
          setMarkets(marketResponse);
        }
      })
      .catch((caughtError: unknown) => {
        if (isCurrent) {
          setError(getMarketError(caughtError, "Unable to load markets."));
        }
      })
      .finally(() => {
        if (isCurrent) {
          setIsMarketsLoading(false);
        }
      });

    return () => {
      isCurrent = false;
    };
  }, [category, isLoading, status]);

  const categories = useMemo(() => {
    return Array.from(new Set(markets.map((market) => market.category))).sort();
  }, [markets]);

  if (isLoading || isMarketsLoading) {
    return (
      <MarketsShell title="Markets" description="Loading market data..." />
    );
  }

  return (
    <section className="mx-auto w-full max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
      <div className="rounded-2xl border border-border bg-surface p-4 sm:p-6">
        <div className="flex flex-col gap-4 border-b border-border pb-5 lg:flex-row lg:items-end lg:justify-between">
          <div>
            <p className="text-sm text-secondary">Prediction Markets</p>
            <h1 className="mt-1 text-3xl font-bold text-text">Markets</h1>
          </div>

          <div className="grid gap-3 sm:grid-cols-2">
            <label className="block">
              <span className="text-xs text-secondary">Status</span>
              <select
                value={status}
                onChange={(event) =>
                  setStatus(event.target.value as MarketStatus)
                }
                className="mt-1 h-10 w-full rounded-lg border border-border bg-card px-3 text-sm text-text outline-none focus:border-accent focus:ring-2 focus:ring-accent/25"
              >
                {MARKET_STATUSES.map((marketStatus) => (
                  <option key={marketStatus} value={marketStatus}>
                    {marketStatus}
                  </option>
                ))}
              </select>
            </label>

            <label className="block">
              <span className="text-xs text-secondary">Category</span>
              <input
                list="market-categories"
                value={category}
                onChange={(event) => setCategory(event.target.value)}
                placeholder="All categories"
                className="mt-1 h-10 w-full rounded-lg border border-border bg-card px-3 text-sm text-text outline-none placeholder:text-secondary focus:border-accent focus:ring-2 focus:ring-accent/25"
              />
              <datalist id="market-categories">
                {categories.map((marketCategory) => (
                  <option key={marketCategory} value={marketCategory} />
                ))}
              </datalist>
            </label>
          </div>
        </div>

        {error ? (
          <div className="mt-5 rounded-xl border border-border bg-card p-4 text-sm text-secondary">
            {error}
          </div>
        ) : null}

        <div className="mt-5 grid gap-4 lg:grid-cols-2">
          {markets.map((market) => (
            <Link
              key={market.id}
              href={`/markets/${market.id}`}
              className="group rounded-2xl border border-border bg-card p-5 transition hover:border-accent focus:outline-none focus:ring-2 focus:ring-accent focus:ring-offset-2 focus:ring-offset-surface"
            >
              <div className="flex items-start justify-between gap-4">
                <span className="rounded-full border border-border bg-surface px-3 py-1 text-xs text-secondary">
                  {market.category}
                </span>
                <span
                  className={`rounded-full border px-3 py-1 text-xs font-semibold ${getStatusClassName(market.status)}`}
                >
                  {market.status}
                </span>
              </div>

              <h2 className="mt-4 text-lg font-semibold text-text group-hover:text-accent">
                {market.title}
              </h2>
              <p className="mt-2 line-clamp-2 text-sm leading-6 text-secondary">
                {market.description}
              </p>

              <div className="mt-5 grid grid-cols-2 gap-3 border-t border-border pt-4 text-sm">
                <MarketMeta
                  label="Starts"
                  value={formatDateTime(market.start_at)}
                />
                <MarketMeta
                  label="Closes"
                  value={formatDateTime(market.close_at)}
                />
              </div>
            </Link>
          ))}
        </div>

        {!error && markets.length === 0 ? (
          <div className="mt-5 rounded-xl border border-border bg-card p-6 text-center text-sm text-secondary">
            No markets match the current filters.
          </div>
        ) : null}
      </div>
    </section>
  );
}

function MarketMeta({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <p className="text-xs text-secondary">{label}</p>
      <p className="mt-1 text-sm font-medium text-text">{value}</p>
    </div>
  );
}

function MarketsShell({
  title,
  description,
  actionHref,
  actionLabel,
}: {
  title: string;
  description: string;
  actionHref?: string;
  actionLabel?: string;
}) {
  return (
    <section className="mx-auto w-full max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
      <div className="rounded-2xl border border-border bg-surface p-6">
        <h1 className="text-3xl font-bold text-text">{title}</h1>
        <p className="mt-2 text-sm text-secondary">{description}</p>
        {actionHref && actionLabel ? (
          <Link
            href={actionHref}
            className="mt-5 inline-flex h-11 items-center justify-center rounded-lg bg-accent px-4 text-sm font-semibold text-text transition hover:brightness-110"
          >
            {actionLabel}
          </Link>
        ) : null}
      </div>
    </section>
  );
}

function getMarketError(error: unknown, fallback: string) {
  if (error instanceof ApiError) {
    return error.message;
  }

  return fallback;
}
