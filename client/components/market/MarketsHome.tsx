"use client";

import Link from "next/link";
import { useEffect, useState } from "react";
import { useAuth } from "@/components/auth/AuthProvider";
import {
  formatDateTime,
  getStatusClassName,
} from "@/components/market/market-format";
import { ApiError } from "@/lib/api/http";
import { getMarkets } from "@/lib/market/market-api";
import type { Market, MarketsQuery } from "@/lib/market/types";

type MarketSortField = Required<
  Pick<MarketsQuery, "order_field">
>["order_field"];
type MarketSortDirection = Required<Pick<MarketsQuery, "order_by">>["order_by"];

type MarketsHomeQuery = {
  order_field: MarketSortField;
  order_by: MarketSortDirection;
  close_after: string;
  close_before: string;
  limit: number;
  skip: number;
};

const DEFAULT_MARKETS_QUERY: MarketsHomeQuery = {
  order_field: "close_at",
  order_by: "ASC",
  close_after: "",
  close_before: "",
  limit: 9,
  skip: 0,
};

const SORT_FIELD_OPTIONS: Array<{ label: string; value: MarketSortField }> = [
  { label: "Closes at", value: "close_at" },
  { label: "Starts at", value: "start_at" },
];

const SORT_DIRECTION_OPTIONS: Array<{
  label: string;
  value: MarketSortDirection;
}> = [
  { label: "Ascending", value: "ASC" },
  { label: "Descending", value: "DESC" },
];

const MARKET_LIMIT_OPTIONS = [6, 9, 12, 18];

const QUERY_KEYS_THAT_RESET_SKIP: Array<keyof MarketsHomeQuery> = [
  "order_field",
  "order_by",
  "close_after",
  "close_before",
  "limit",
];

export function MarketsHome() {
  const { isLoading } = useAuth();
  const [markets, setMarkets] = useState<Market[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [isMarketsLoading, setIsMarketsLoading] = useState(true);
  const [query, setQuery] = useState(DEFAULT_MARKETS_QUERY);

  useEffect(() => {
    if (isLoading) {
      return;
    }

    let isCurrent = true;
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setIsMarketsLoading(true);
    setMarkets([]);
    setError(null);

    getMarkets(query)
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
  }, [isLoading, query]);

  function updateQuery<Key extends keyof MarketsHomeQuery>(
    key: Key,
    value: MarketsHomeQuery[Key]
  ) {
    setIsMarketsLoading(true);
    setQuery((currentQuery) => ({
      ...currentQuery,
      [key]: value,
      ...(QUERY_KEYS_THAT_RESET_SKIP.includes(key) ? { skip: 0 } : {}),
    }));
  }

  function handlePreviousPage() {
    setIsMarketsLoading(true);
    setQuery((currentQuery) => ({
      ...currentQuery,
      skip: Math.max(0, currentQuery.skip - currentQuery.limit),
    }));
  }

  function handleNextPage() {
    setIsMarketsLoading(true);
    setQuery((currentQuery) => ({
      ...currentQuery,
      skip: currentQuery.skip + currentQuery.limit,
    }));
  }

  function handleResetFilters() {
    setIsMarketsLoading(true);
    setQuery(DEFAULT_MARKETS_QUERY);
  }

  if (isLoading || isMarketsLoading) {
    return <MarketsSkeleton />;
  }

  return (
    <section className="mx-auto w-full max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
      {/* Filters Grid */}
      <div className="rounded-2xl border border-border bg-surface p-4 sm:p-6">
        <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-6">
          <div className="block">
            <span className="text-xs font-medium uppercase text-secondary">
              Reset Filters
            </span>
            <button
              type="button"
              onClick={handleResetFilters}
              className="mt-2 h-10 w-full rounded-lg border border-border bg-card px-4 text-sm font-semibold text-text transition hover:border-accent disabled:opacity-40"
            >
              Reset
            </button>
          </div>
          <SelectField
            id="markets-sort-field"
            label="Sort by"
            value={query.order_field}
            onChange={(value) => updateQuery("order_field", value)}
            options={SORT_FIELD_OPTIONS}
          />
          <SelectField
            id="markets-sort-direction"
            label="Direction"
            value={query.order_by}
            onChange={(value) => updateQuery("order_by", value)}
            options={SORT_DIRECTION_OPTIONS}
          />
          <DateField
            id="markets-close-after"
            label="Close after"
            value={query.close_after ? query.close_after.split("T")[0] : ""}
            onChange={(value) =>
              updateQuery("close_after", value ? `${value}T00:00:00Z` : "")
            }
          />
          <DateField
            id="markets-close-before"
            label="Close before"
            value={query.close_before ? query.close_before.split("T")[0] : ""}
            onChange={(value) =>
              updateQuery("close_before", value ? `${value}T00:00:00Z` : "")
            }
          />
          <SelectField
            id="markets-limit"
            label="Limit"
            value={String(query.limit)}
            onChange={(value) => updateQuery("limit", Number(value))}
            options={MARKET_LIMIT_OPTIONS.map((value) => ({
              label: String(value),
              value: String(value),
            }))}
          />
        </div>
      </div>

      {/* Error State */}
      {error ? (
        <div className="mt-6 rounded-xl border border-border bg-card p-4 text-sm text-secondary">
          {error}
        </div>
      ) : null}

      {/* Markets Grid */}
      <div className="mt-6 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
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

      {/* Empty State */}
      {!error && markets.length === 0 ? (
        <div className="mt-6 rounded-xl border border-border bg-card p-6 text-center text-sm text-secondary">
          No markets match the current filters.
        </div>
      ) : null}

      {/* Pagination Controls */}
      {/* 
        Show pagination if:
        1. We received a full page of results (markets.length >= query.limit), meaning there might be more pages.
        2. OR we are not on the first page (query.skip > 0), so the user needs the "Previous" button to go back.
      */}
      {(markets.length >= query.limit || query.skip > 0) && (
        <div className="mt-6 flex justify-center gap-2">
          <button
            type="button"
            onClick={handlePreviousPage}
            disabled={query.skip === 0}
            className="h-10 rounded-lg border border-border bg-card px-4 text-sm font-semibold text-text transition hover:border-accent disabled:opacity-40"
          >
            Previous
          </button>
          <button
            type="button"
            onClick={handleNextPage}
            disabled={markets.length < query.limit}
            className="h-10 rounded-lg border border-border bg-card px-4 text-sm font-semibold text-text transition hover:border-accent disabled:opacity-40"
          >
            Next
          </button>
        </div>
      )}
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

function SelectField<T extends string>({
  id,
  label,
  value,
  options,
  onChange,
}: {
  id: string;
  label: string;
  value: T;
  options: Array<{ label: string; value: T }>;
  onChange: (value: T) => void;
}) {
  return (
    <label htmlFor={id} className="block">
      <span className="text-xs font-medium uppercase text-secondary">
        {label}
      </span>
      <select
        id={id}
        value={value}
        onChange={(event) => onChange(event.target.value as T)}
        className="mt-2 h-10 w-full rounded-lg border border-border bg-card px-3 text-sm text-text outline-none transition focus:border-accent focus:ring-2 focus:ring-accent/25"
      >
        {options.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
    </label>
  );
}

function DateField({
  id,
  label,
  value,
  onChange,
}: {
  id: string;
  label: string;
  value: string;
  onChange: (value: string) => void;
}) {
  return (
    <label htmlFor={id} className="block">
      <span className="text-xs font-medium uppercase text-secondary">
        {label}
      </span>
      <input
        id={id}
        type="date"
        value={value}
        onChange={(event) => onChange(event.target.value)}
        className="mt-2 h-10 w-full rounded-lg border border-border bg-card px-3 text-sm text-text outline-none transition placeholder:text-secondary focus:border-accent focus:ring-2 focus:ring-accent/25"
      />
    </label>
  );
}

function MarketsSkeleton() {
  return (
    <section className="mx-auto w-full max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
      <div className="rounded-2xl border border-border bg-surface p-4 sm:p-6">
        <div className="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
          <div>
            <div className="h-4 w-32 animate-pulse rounded bg-card" />
            <div className="mt-2 h-8 w-48 animate-pulse rounded bg-card" />
            <div className="mt-2 h-4 w-96 max-w-full animate-pulse rounded bg-card" />
          </div>
          <div className="h-10 w-36 animate-pulse rounded-lg bg-card" />
        </div>

        <div className="mt-6 grid gap-3 sm:grid-cols-2 lg:grid-cols-6">
          <div className="h-[74px] animate-pulse rounded-lg bg-card" />
          <div className="h-[74px] animate-pulse rounded-lg bg-card" />
          <div className="h-[74px] animate-pulse rounded-lg bg-card" />
          <div className="h-[74px] animate-pulse rounded-lg bg-card" />
          <div className="h-[74px] animate-pulse rounded-lg bg-card" />
          <div className="h-[74px] animate-pulse rounded-lg bg-card" />
        </div>
      </div>

      <div className="mt-6 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {[...Array(6)].map((_, i) => (
          <div key={i} className="rounded-2xl border border-border bg-card p-5">
            <div className="flex items-start justify-between gap-4">
              <div className="h-6 w-20 animate-pulse rounded-full bg-surface" />
              <div className="h-6 w-16 animate-pulse rounded-full bg-surface" />
            </div>
            <div className="mt-4 h-6 w-3/4 animate-pulse rounded bg-surface" />
            <div className="mt-2 h-4 w-full animate-pulse rounded bg-surface" />
            <div className="mt-1 h-4 w-2/3 animate-pulse rounded bg-surface" />
            <div className="mt-5 grid grid-cols-2 gap-3 border-t border-border pt-4">
              <div className="h-4 w-16 animate-pulse rounded bg-surface" />
              <div className="h-4 w-16 animate-pulse rounded bg-surface" />
            </div>
          </div>
        ))}
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
