"use client";

import Link from "next/link";
import { useEffect, useRef, useState } from "react";
import { useWindowVirtualizer } from "@tanstack/react-virtual";
import { useAuth } from "@/components/auth/AuthProvider";
import { ApiError } from "@/lib/api/http";
import { getMarkets } from "@/lib/market/market-api";
import type { Market } from "@/lib/market/types";

const MARKETS_LIMIT = 9;

export function MarketsHome() {
  const { isLoading } = useAuth();
  const [markets, setMarkets] = useState<Market[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [isMarketsLoading, setIsMarketsLoading] = useState(true);
  const [nextSkip, setNextSkip] = useState(0);
  const [hasMoreMarkets, setHasMoreMarkets] = useState(true);
  const [columnCount, setColumnCount] = useState(1);
  const isLoadingMoreRef = useRef(false);

  const marketRows: Market[][] = [];
  for (let index = 0; index < markets.length; index += columnCount) {
    marketRows.push(markets.slice(index, index + columnCount));
  }

  const rowVirtualizer = useWindowVirtualizer({
    count: marketRows.length,
    estimateSize: () => 280,
    gap: 16,
    overscan: 2,
  });

  useEffect(() => {
    if (isLoading) {
      return;
    }

    let isCurrent = true;
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setIsMarketsLoading(true);
    isLoadingMoreRef.current = false;
    setError(null);
    setNextSkip(MARKETS_LIMIT);
    setHasMoreMarkets(true);

    getMarkets({
      order_field: "close_at",
      order_by: "DESC",
      limit: MARKETS_LIMIT,
      skip: 0,
    })
      .then((marketResponse) => {
        if (isCurrent) {
          setMarkets(marketResponse);
          setHasMoreMarkets(marketResponse.length === MARKETS_LIMIT);
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
  }, [isLoading]);

  useEffect(() => {
    function updateColumnCount() {
      setColumnCount(
        window.innerWidth >= 1024 ? 3 : window.innerWidth >= 640 ? 2 : 1
      );
    }

    updateColumnCount();
    window.addEventListener("resize", updateColumnCount);
    return () => window.removeEventListener("resize", updateColumnCount);
  }, []);

  useEffect(() => {
    const lastVirtualRow = rowVirtualizer.getVirtualItems().at(-1);
    if (
      !lastVirtualRow ||
      lastVirtualRow.index < marketRows.length - 2 ||
      isLoadingMoreRef.current ||
      !hasMoreMarkets
    ) {
      return;
    }

    isLoadingMoreRef.current = true;
    let isCurrent = true;
    setIsMarketsLoading(true);

    getMarkets({
      order_field: "close_at",
      order_by: "ASC",
      limit: MARKETS_LIMIT,
      skip: nextSkip,
    })
      .then((marketResponse) => {
        if (isCurrent) {
          setMarkets((currentMarkets) => [
            ...currentMarkets,
            ...marketResponse,
          ]);
          setNextSkip((currentSkip) => currentSkip + MARKETS_LIMIT);
          setHasMoreMarkets(marketResponse.length === MARKETS_LIMIT);
        }
      })
      .catch((caughtError: unknown) => {
        if (isCurrent) {
          setError(getMarketError(caughtError, "Unable to load more markets."));
        }
      })
      .finally(() => {
        if (isCurrent) {
          isLoadingMoreRef.current = false;
          setIsMarketsLoading(false);
        }
      });

    return () => {
      isCurrent = false;
    };
  }, [hasMoreMarkets, marketRows.length, nextSkip, rowVirtualizer]);

  if (isLoading || isMarketsLoading) {
    return <MarketsSkeleton />;
  }

  return (
    <section className="mx-auto w-full max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
      {/* Error State */}
      {error ? (
        <div className="mt-6 rounded-xl border border-border bg-card p-4 text-sm text-secondary">
          {error}
        </div>
      ) : null}

      {/* Virtualized Markets Grid */}
      <div
        className="relative mt-6"
        style={{ height: `${rowVirtualizer.getTotalSize()}px` }}
      >
        {rowVirtualizer.getVirtualItems().map((virtualRow) => {
          const row = marketRows[virtualRow.index];

          return (
            <div
              key={virtualRow.key}
              data-index={virtualRow.index}
              ref={rowVirtualizer.measureElement}
              className="absolute left-0 top-0 grid w-full gap-x-4 sm:grid-cols-2 lg:grid-cols-3"
              style={{ transform: `translateY(${virtualRow.start}px)` }}
            >
              {row.map((market) => (
                <MarketCard key={market.id} market={market} />
              ))}
            </div>
          );
        })}
      </div>

      {/* Empty State */}
      {!error && markets.length === 0 ? (
        <div className="mt-6 rounded-xl border border-border bg-card p-6 text-center text-sm text-secondary">
          No markets are available.
        </div>
      ) : null}

      {isMarketsLoading && markets.length > 0 ? (
        <p className="mt-6 text-center text-sm text-secondary">
          Loading more markets...
        </p>
      ) : null}
    </section>
  );
}

function MarketCard({ market }: { market: Market }) {
  return (
    <Link
      href={`/markets/${market.id}`}
      className="group rounded-2xl border border-border bg-card p-5 transition hover:border-accent focus:outline-none focus:ring-2 focus:ring-accent focus:ring-offset-2 focus:ring-offset-surface"
    >
      <h2 className="text-lg font-semibold text-text group-hover:text-accent">
        {market.title}
      </h2>
      <p className="mt-2 line-clamp-2 text-sm leading-6 text-secondary">
        {market.description}
      </p>

      <div className="mt-5 grid grid-cols-2 gap-3 border-t border-border pt-4 text-sm">
        <MarketMeta label="Category" value={market.category} />
        <MarketMeta
          label="Closes"
          value={formatMarketDateTime(market.close_at)}
        />
      </div>
    </Link>
  );
}

function MarketMeta({
  label,
  value,
}: {
  label: string;
  value: { time: string; date: string } | string;
}) {
  return (
    <div>
      <p className="text-xs text-secondary">{label}</p>
      {typeof value === "string" ? (
        <p className="mt-1 text-sm font-medium text-text">{value}</p>
      ) : (
        <>
          <p className="mt-1 text-sm font-medium text-text">{value.time}</p>
          <p className="text-xs text-secondary">{value.date}</p>
        </>
      )}
    </div>
  );
}

function formatMarketDateTime(value: string) {
  const date = new Date(value);

  if (Number.isNaN(date.getTime())) {
    return { time: "Unknown", date: "Unknown" };
  }

  return {
    time: new Intl.DateTimeFormat("en-US", { timeStyle: "short" }).format(date),
    date: new Intl.DateTimeFormat("en-US", { dateStyle: "medium" }).format(
      date
    ),
  };
}

function MarketsSkeleton() {
  return (
    <section className="mx-auto w-full max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
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
