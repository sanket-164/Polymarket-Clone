"use client";

import Link from "next/link";
import { useEffect, useState } from "react";
import { useAuth } from "@/components/auth/AuthProvider";
import { ApiError } from "@/lib/api/http";
import { getHoldings } from "@/lib/holding/holding-api";
import type { Holding, HoldingsQuery } from "@/lib/holding/types";

const DEFAULT_HOLDINGS_QUERY: Required<HoldingsQuery> = {
  order_field: "shares",
  order_by: "DESC",
  limit: 10,
  skip: 0,
};

export function HoldingsPage() {
  const { isAuthenticated, isLoading } = useAuth();
  const [holdings, setHoldings] = useState<Holding[]>([]);
  const [isHoldingsLoading, setIsHoldingsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [query, setQuery] = useState(DEFAULT_HOLDINGS_QUERY);

  useEffect(() => {
    if (isLoading) return;

    if (!isAuthenticated) {
      // eslint-disable-next-line react-hooks/set-state-in-effect
      setHoldings([]);
      setError(null);
      setIsHoldingsLoading(false);
      return;
    }

    let isCurrent = true;
    setIsHoldingsLoading(true);
    setError(null);

    getHoldings(query)
      .then((response) => {
        if (isCurrent) {
          setHoldings(response);
        }
      })
      .catch((caughtError: unknown) => {
        if (isCurrent) {
          setError(
            getHoldingsError(caughtError, "Unable to load your holdings.")
          );
        }
      })
      .finally(() => {
        if (isCurrent) {
          setIsHoldingsLoading(false);
        }
      });

    return () => {
      isCurrent = false;
    };
  }, [isAuthenticated, isLoading, query]);

  function handleFilterChange(
    key: keyof Required<HoldingsQuery>,
    value: string
  ) {
    setIsHoldingsLoading(true);
    setQuery((currentQuery) => ({
      ...currentQuery,
      [key]: key === "limit" || key === "skip" ? Number(value) : value,
      ...(key === "order_field" ? { skip: 0 } : null),
    }));
  }

  function handlePrevious() {
    setIsHoldingsLoading(true);
    setQuery((currentQuery) => ({
      ...currentQuery,
      skip: Math.max(0, currentQuery.skip - currentQuery.limit),
    }));
  }

  function handleNext() {
    setIsHoldingsLoading(true);
    setQuery((currentQuery) => ({
      ...currentQuery,
      skip: currentQuery.skip + currentQuery.limit,
    }));
  }

  if (isLoading) {
    return (
      <HoldingsShell
        title="Loading holdings"
        description="Checking your session..."
      />
    );
  }

  if (!isAuthenticated) {
    return (
      <HoldingsShell
        title="Holdings"
        description="Sign in to view your portfolio holdings."
        actionHref="/login"
        actionLabel="Log in"
      />
    );
  }

  return (
    <section className="mx-auto w-full max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
      <div className="rounded-2xl border border-border bg-surface p-4 sm:p-6">
        <div className="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
          <div>
            <p className="text-sm text-secondary">Portfolio</p>
            <h1 className="mt-1 text-2xl font-bold text-text sm:text-3xl">
              Holdings
            </h1>
            <p className="mt-2 text-sm text-secondary">
              Review the markets and outcomes you currently hold.
            </p>
          </div>
          <div className="grid gap-2 sm:grid-cols-3 lg:w-[480px]">
            <SelectField
              id="holdings-order-field"
              label="Sort by"
              value={query.order_field}
              onChange={(value) => handleFilterChange("order_field", value)}
              options={[
                { label: "Shares", value: "shares" },
                { label: "Created", value: "created_at" },
              ]}
            />
            <SelectField
              id="holdings-order-direction"
              label="Direction"
              value={query.order_by}
              onChange={(value) => handleFilterChange("order_by", value)}
              options={[
                { label: "Descending", value: "DESC" },
                { label: "Ascending", value: "ASC" },
              ]}
            />
            <SelectField
              id="holdings-limit"
              label="Limit"
              value={String(query.limit)}
              onChange={(value) => handleFilterChange("limit", value)}
              options={[
                { label: "5", value: "5" },
                { label: "10", value: "10" },
                { label: "15", value: "15" },
              ]}
            />
          </div>
        </div>

        {error ? (
          <div className="mt-6 rounded-xl border border-accent/40 bg-card p-4 text-sm text-secondary">
            {error}
          </div>
        ) : null}

        <div className="mt-6 overflow-x-auto rounded-xl border border-border bg-card">
          <table className="w-full min-w-[860px] border-collapse text-left text-sm">
            <thead className="bg-surface text-xs uppercase text-secondary">
              <tr>
                <th className="border-b border-border px-3 py-3 font-medium">
                  Market
                </th>
                <th className="border-b border-border px-3 py-3 font-medium">
                  Outcome
                </th>
                <th className="border-b border-border px-3 py-3 font-medium">
                  Shares
                </th>
                <th className="border-b border-border px-3 py-3 font-medium">
                  Locked
                </th>
                <th className="border-b border-border px-3 py-3 font-medium">
                  Created at
                </th>
              </tr>
            </thead>
            <tbody>
              {isHoldingsLoading ? (
                <SkeletonHoldingRows />
              ) : holdings.length > 0 ? (
                holdings.map((holding) => (
                  <tr key={holding.id} className="transition hover:bg-surface">
                    <td className="border-b border-border px-3 py-3">
                      <Link
                        href={`/markets/${holding.market.id}`}
                        className="inline-flex items-center gap-2 font-semibold text-text transition hover:text-accent"
                      >
                        <span>{holding.market.title}</span>
                        <span className="rounded-full border border-border bg-surface px-2.5 py-1 text-[11px] font-semibold uppercase tracking-wide text-secondary">
                          {holding.market.status}
                        </span>
                      </Link>
                      <div className="mt-1 text-xs text-secondary">
                        {holding.market.category}
                      </div>
                    </td>
                    <td className="border-b border-border px-3 py-3">
                      <div className="font-medium text-text">
                        {holding.outcome.label}
                      </div>
                    </td>
                    <td className="border-b border-border px-3 py-3 font-mono text-text">
                      {formatShares(holding.shares)}
                    </td>
                    <td className="border-b border-border px-3 py-3 font-mono text-secondary">
                      {formatShares(holding.locked_shares)}
                    </td>
                    <td className="border-b border-border px-3 py-3 text-secondary">
                      {formatDateTime(holding.created_at)}
                    </td>
                  </tr>
                ))
              ) : (
                <tr>
                  <td
                    colSpan={5}
                    className="px-3 py-6 text-center text-secondary"
                  >
                    No holdings found.
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>

        <div className="mt-4 flex flex-col gap-2 sm:flex-row sm:justify-end">
          <button
            type="button"
            disabled={query.skip === 0 || isHoldingsLoading}
            onClick={handlePrevious}
            className="h-10 rounded-lg border border-border bg-surface px-4 text-sm font-semibold text-text transition hover:border-accent disabled:opacity-40"
          >
            Previous
          </button>
          <button
            type="button"
            disabled={holdings.length < query.limit || isHoldingsLoading}
            onClick={handleNext}
            className="h-10 rounded-lg border border-border bg-surface px-4 text-sm font-semibold text-text transition hover:border-accent disabled:opacity-40"
          >
            Next
          </button>
        </div>
      </div>
    </section>
  );
}

type SelectFieldProps = {
  id: string;
  label: string;
  value: string;
  options: Array<{ label: string; value: string }>;
  onChange: (value: string) => void;
};

function SelectField({
  id,
  label,
  value,
  options,
  onChange,
}: SelectFieldProps) {
  return (
    <label htmlFor={id} className="block">
      <span className="text-xs font-medium uppercase text-secondary">
        {label}
      </span>
      <select
        id={id}
        value={value}
        onChange={(event) => onChange(event.target.value)}
        className="mt-2 h-10 w-full rounded-lg border border-border bg-surface px-3 text-sm text-text outline-none transition focus:border-accent focus:ring-2 focus:ring-accent/25"
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

function HoldingsShell({
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
        <p className="text-sm text-secondary">Portfolio</p>
        <h1 className="mt-2 text-3xl font-bold text-text">{title}</h1>
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

function formatShares(value: string | number | null | undefined) {
  if (value === null || value === undefined || Number.isNaN(Number(value))) {
    return "--";
  }

  return new Intl.NumberFormat("en-US", {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(Number(value));
}

function formatDateTime(value: string | null | undefined) {
  if (!value) {
    return "Unavailable";
  }

  return new Intl.DateTimeFormat("en-US", {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(value));
}

function getHoldingsError(caughtError: unknown, fallback: string) {
  if (caughtError instanceof ApiError) {
    return caughtError.message;
  }

  if (caughtError instanceof Error) {
    return caughtError.message;
  }

  return fallback;
}

function SkeletonHoldingRows() {
  return (
    <>
      {Array.from({ length: 5 }).map((_, index) => (
        <tr key={index} className="animate-pulse">
          <td className="border-b border-border px-3 py-3">
            <div className="space-y-2">
              <div className="h-4 w-48 rounded bg-surface" />
              <div className="h-3 w-24 rounded bg-surface" />
            </div>
          </td>
          <td className="border-b border-border px-3 py-3">
            <div className="h-4 w-32 rounded bg-surface" />
          </td>
          <td className="border-b border-border px-3 py-3">
            <div className="h-4 w-20 rounded bg-surface" />
          </td>
          <td className="border-b border-border px-3 py-3">
            <div className="h-4 w-20 rounded bg-surface" />
          </td>
          <td className="border-b border-border px-3 py-3">
            <div className="h-4 w-28 rounded bg-surface" />
          </td>
        </tr>
      ))}
    </>
  );
}
