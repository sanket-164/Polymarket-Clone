"use client";

import Link from "next/link";
import { useEffect, useRef, useState } from "react";
import { useAuth } from "@/components/auth/AuthProvider";
import {
  formatDateTime,
  formatShares,
  getStatusClassName,
} from "@/components/market/market-format";
import { ApiError } from "@/lib/api/http";
import { getMarketDetails, getMarketSnapshot } from "@/lib/market/market-api";
import type { MarketDetails, MarketSnapshot } from "@/lib/market/types";
import { MarketOrderBook } from "@/components/market/MarketOrderBook";
import { MarketPriceGraph } from "@/components/market/MarketPriceGraph";
import { LimitOrderForm } from "@/components/order/LimitOrderForm";
import { useOrderbookWebSocket } from "@/hooks/useOrderbookWebSocket";

export function MarketDetailsPage({ marketId }: { marketId: string }) {
  const { isLoading } = useAuth();
  const [market, setMarket] = useState<MarketDetails | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isMarketLoading, setIsMarketLoading] = useState(true);

  useEffect(() => {
    if (isLoading) return;

    let isCurrent = true;
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setIsMarketLoading(true);
    setError(null);

    getMarketDetails(marketId)
      .then((marketResponse) => {
        if (!isCurrent) return;
        setMarket(marketResponse);
      })
      .catch((caughtError: unknown) => {
        if (isCurrent)
          setError(
            getMarketError(caughtError, "Unable to load market details.")
          );
      })
      .finally(() => {
        if (isCurrent) setIsMarketLoading(false);
      });

    return () => {
      isCurrent = false;
    };
  }, [isLoading, marketId]);

  const { snapshots, currentPrices } = useOrderbookWebSocket(
    marketId,
    () => getMarketSnapshot(marketId),
    !isLoading && !!market
  );

  // Maintain price history for the real-time graph
  const [priceHistory, setPriceHistory] = useState<
    { time: string; first: number; second: number }[]
  >([]);
  const lastUpdateRef = useRef<number>(0);

  // Initialize history when market loads
  useEffect(() => {
    if (market && priceHistory.length === 0) {
      // eslint-disable-next-line react-hooks/set-state-in-effect
      setPriceHistory([
        {
          time: new Date().toLocaleTimeString(),
          first: Number(market.first_outcome.current_price),
          second: Number(market.second_outcome.current_price),
        },
      ]);
    }
  }, [market, priceHistory.length]);

  // Update history when currentPrices change (throttled to avoid overwhelming the chart)
  useEffect(() => {
    if (!market || Object.keys(currentPrices).length === 0) return;

    const now = Date.now();
    // Throttle updates to once per 500ms for smooth performance
    if (now - lastUpdateRef.current < 500) return;
    lastUpdateRef.current = now;

    const firstPrice = Number(
      currentPrices[market.first_outcome.id] ??
        market.first_outcome.current_price
    );
    const secondPrice = Number(
      currentPrices[market.second_outcome.id] ??
        market.second_outcome.current_price
    );

    setPriceHistory((prev) => {
      const newPoint = {
        time: new Date().toLocaleTimeString(),
        first: firstPrice,
        second: secondPrice,
      };
      const newHistory = [...prev, newPoint];
      // Keep last 60 points to maintain performance and keep the chart readable
      if (newHistory.length > 60) {
        return newHistory.slice(newHistory.length - 60);
      }
      return newHistory;
    });
  }, [currentPrices, market]);

  if (isLoading || isMarketLoading) {
    return (
      <MarketShell title="Market" description="Loading market details..." />
    );
  }

  if (error) {
    return (
      <section className="mx-auto w-full max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
        <div className="rounded-2xl border border-border bg-surface p-6">
          <div className="flex items-center justify-between gap-4">
            <div>
              <p className="text-sm text-secondary">Prediction market</p>
              <h1 className="mt-1 text-3xl font-bold text-text">
                Market details
              </h1>
            </div>
            <Link
              href="/"
              className="rounded-lg border border-border bg-card px-4 py-2 text-sm font-semibold text-text transition hover:border-accent"
            >
              Back to markets
            </Link>
          </div>
          <div className="mt-5 rounded-xl border border-border bg-card p-4 text-sm text-secondary">
            {error}
          </div>
        </div>
      </section>
    );
  }

  if (!market) {
    return (
      <MarketShell
        title="Market not found"
        description="The requested market could not be loaded."
        actionHref="/"
        actionLabel="Back to markets"
      />
    );
  }

  // Construct a MarketSnapshot-like object for MarketOrderBook compatibility
  const currentSnapshot: MarketSnapshot | null =
    snapshots.length > 0
      ? {
          market_id: marketId,
          snapshot: snapshots,
          updated_at: new Date().getTime(),
        }
      : null;

  return (
    <section className="mx-auto w-full max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
      {/* Main Grid */}
      <div className="grid gap-6 lg:grid-cols-3">
        {/* Left Column: Chart & Order Book */}
        <div className="space-y-6 lg:col-span-2">
          <div className="rounded-2xl border border-border bg-surface p-6">
            <div className="mb-6 flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
              <div className="space-y-2">
                <Link
                  href="/"
                  className="inline-flex items-center gap-2 text-sm font-medium text-secondary transition hover:text-text"
                >
                  <svg
                    className="size-4"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                    strokeWidth={2}
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      d="M15 19l-7-7 7-7"
                    />
                  </svg>
                  Back to markets
                </Link>
                <h1 className="text-2xl font-bold tracking-tight text-text sm:text-3xl">
                  {market.title}
                </h1>
                <p className="max-w-3xl text-sm text-secondary">
                  {market.description}
                </p>
              </div>
              <div className="flex flex-wrap items-center gap-2">
                <span className="rounded-full border border-border bg-card px-3 py-1 text-xs font-medium text-secondary">
                  {market.category}
                </span>
                <span
                  className={`rounded-full border px-3 py-1 text-xs font-semibold ${getStatusClassName(market.status)}`}
                >
                  {market.status}
                </span>
              </div>
            </div>

            {/* Price Graph */}
            <MarketPriceGraph
              firstOutcome={market.first_outcome}
              secondOutcome={market.second_outcome}
              priceHistory={priceHistory}
            />
          </div>

          {/* Order Book */}
          {currentSnapshot && (
            <MarketOrderBook
              snapshot={currentSnapshot}
              firstOutcome={market.first_outcome}
              secondOutcome={market.second_outcome}
              currentPrices={currentPrices}
            />
          )}
        </div>

        {/* Right Column: Limit Order Form & Market Details */}
        <div className="border border-border bg-surface rounded-2xl lg:col-span-1">
          {/* Limit Order Form */}
          <LimitOrderForm
            marketId={marketId}
            firstOutcome={market.first_outcome}
            secondOutcome={market.second_outcome}
            currentPrices={currentPrices}
            onSuccess={() => {
              // Optionally refresh orderbook or show notification
            }}
          />

          <div className="space-y-3 p-6">
            <DetailRow
              label={`Available ${market.first_outcome.label} Shares`}
              value={formatShares(Number(market.first_outcome.total_shares))}
            />
            <DetailRow
              label={`Available ${market.second_outcome.label} Shares`}
              value={formatShares(Number(market.second_outcome.total_shares))}
            />
            <DetailRow label="Starts" value={formatDateTime(market.start_at)} />
            <DetailRow label="Closes" value={formatDateTime(market.close_at)} />
          </div>
        </div>
      </div>
    </section>
  );
}

function DetailRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between border-b border-border/50 pb-2 last:border-0 last:pb-0">
      <span className="text-sm text-secondary">{label}</span>
      <span className="text-sm font-medium text-text tabular-nums">
        {value}
      </span>
    </div>
  );
}

function MarketShell({
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
  if (error instanceof ApiError) return error.message;
  return fallback;
}
