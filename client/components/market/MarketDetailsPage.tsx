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
import { getOrders } from "@/lib/order/order-api";
import type { Order, OrdersQuery } from "@/lib/order/types";

const DEFAULT_MARKET_ORDER_QUERY: Required<OrdersQuery> = {
  market_id: "",
  order_by: "DESC",
  order_field: "created_at",
  side: "",
  status: "",
  limit: 5,
  skip: 0,
  before: "",
  after: "",
};

export function MarketDetailsPage({ marketId }: { marketId: string }) {
  const { isLoading, isAuthenticated } = useAuth();
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
  const [showMarketOrders, setShowMarketOrders] = useState(false);
  const [marketOrders, setMarketOrders] = useState<Order[]>([]);
  const [marketOrderQuery, setMarketOrderQuery] = useState({
    ...DEFAULT_MARKET_ORDER_QUERY,
    market_id: marketId,
  });
  const [isMarketOrdersLoading, setIsMarketOrdersLoading] = useState(false);
  const [marketOrdersError, setMarketOrdersError] = useState<string | null>(
    null
  );
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

  useEffect(() => {
    if (!isAuthenticated) {
      // eslint-disable-next-line react-hooks/set-state-in-effect
      setShowMarketOrders(false);
      setMarketOrders([]);
      setMarketOrderQuery(DEFAULT_MARKET_ORDER_QUERY);
      setMarketOrdersError(null);
      return;
    }

    if (!showMarketOrders || isLoading) return;

    let isCurrent = true;
    setIsMarketOrdersLoading(true);
    setMarketOrdersError(null);

    getOrders(marketOrderQuery)
      .then((ordersResponse) => {
        if (!isCurrent) return;
        setMarketOrders(
          ordersResponse.filter((order) => order.market_id === marketId)
        );
      })
      .catch((caughtError: unknown) => {
        if (isCurrent) {
          setMarketOrdersError(
            getMarketError(
              caughtError,
              "Unable to load your orders for this market."
            )
          );
        }
      })
      .finally(() => {
        if (isCurrent) setIsMarketOrdersLoading(false);
      });

    return () => {
      isCurrent = false;
    };
  }, [
    isAuthenticated,
    isLoading,
    marketOrderQuery,
    marketId,
    showMarketOrders,
  ]);

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

  function handleMarketOrderFilterChange(
    key: keyof Required<OrdersQuery>,
    value: string
  ) {
    setIsMarketOrdersLoading(true);
    setMarketOrderQuery((currentQuery) => ({
      ...currentQuery,
      [key]: key === "limit" || key === "skip" ? Number(value) : value,
      ...(key === "side" || key === "status" ? { skip: 0 } : null),
    }));
  }

  function handlePreviousMarketOrders() {
    setIsMarketOrdersLoading(true);
    setMarketOrderQuery((currentQuery) => ({
      ...currentQuery,
      skip: Math.max(0, currentQuery.skip - currentQuery.limit),
    }));
  }

  function handleNextMarketOrders() {
    setIsMarketOrdersLoading(true);
    setMarketOrderQuery((currentQuery) => ({
      ...currentQuery,
      skip: currentQuery.skip + currentQuery.limit,
    }));
  }

  if (isLoading || isMarketLoading) {
    return <MarketDetailsSkeleton />;
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

      {isAuthenticated ? (
        <div className="mt-6 rounded-2xl border border-border bg-surface p-6">
          <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
            <div>
              <h2 className="text-xl font-semibold text-text">
                Review your buy and sell activity
              </h2>
            </div>
            <button
              type="button"
              onClick={() => setShowMarketOrders((current) => !current)}
              className="inline-flex h-11 items-center justify-center rounded-lg border border-border bg-card px-4 text-sm font-semibold text-text transition hover:border-accent"
            >
              {showMarketOrders ? "Hide" : "View"}
            </button>
          </div>

          {showMarketOrders ? (
            <div className="mt-6 space-y-4">
              {marketOrdersError ? (
                <div className="rounded-xl border border-accent/40 bg-card p-4 text-sm text-secondary">
                  {marketOrdersError}
                </div>
              ) : null}

              <div className="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
                <div className="grid gap-2 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5">
                  <SelectField
                    id="market-order-side"
                    label="Side"
                    value={marketOrderQuery.side}
                    onChange={(value) =>
                      handleMarketOrderFilterChange("side", value)
                    }
                    options={[
                      { label: "All", value: "" },
                      { label: "Buy", value: "BUY" },
                      { label: "Sell", value: "SELL" },
                    ]}
                  />
                  <SelectField
                    id="market-order-status"
                    label="Status"
                    value={marketOrderQuery.status}
                    onChange={(value) =>
                      handleMarketOrderFilterChange("status", value)
                    }
                    options={[
                      { label: "All", value: "" },
                      { label: "Pending", value: "PENDING" },
                      { label: "Filled", value: "FILLED" },
                      { label: "Partially filled", value: "PARTIAL" },
                      { label: "Cancelled", value: "CANCELLED" },
                      { label: "Expired", value: "EXPIRED" },
                    ]}
                  />
                  <SelectField
                    id="market-order-field"
                    label="Sort by"
                    value={marketOrderQuery.order_field}
                    onChange={(value) =>
                      handleMarketOrderFilterChange("order_field", value)
                    }
                    options={[
                      { label: "Created", value: "created_at" },
                      { label: "Price", value: "price" },
                      { label: "Shares", value: "shares" },
                    ]}
                  />
                  <SelectField
                    id="market-order-direction"
                    label="Direction"
                    value={marketOrderQuery.order_by}
                    onChange={(value) =>
                      handleMarketOrderFilterChange("order_by", value)
                    }
                    options={[
                      { label: "Descending", value: "DESC" },
                      { label: "Ascending", value: "ASC" },
                    ]}
                  />
                  <SelectField
                    id="market-order-limit"
                    label="Limit"
                    value={String(marketOrderQuery.limit)}
                    onChange={(value) =>
                      handleMarketOrderFilterChange("limit", value)
                    }
                    options={[
                      { label: "5", value: "5" },
                      { label: "10", value: "10" },
                      { label: "15", value: "15" },
                    ]}
                  />
                </div>
              </div>

              <div className="overflow-x-auto rounded-xl border border-border bg-card">
                <table className="w-full min-w-[760px] border-collapse text-left text-sm">
                  <thead className="bg-surface text-xs uppercase text-secondary">
                    <tr>
                      <th className="border-b border-border px-3 py-3 font-medium">
                        Side
                      </th>
                      <th className="border-b border-border px-3 py-3 font-medium">
                        Shares
                      </th>
                      <th className="border-b border-border px-3 py-3 font-medium">
                        Price
                      </th>
                      <th className="border-b border-border px-3 py-3 font-medium">
                        Status
                      </th>
                      <th className="border-b border-border px-3 py-3 font-medium">
                        Order ID
                      </th>
                      <th className="border-b border-border px-3 py-3 font-medium">
                        Created
                      </th>
                    </tr>
                  </thead>
                  <tbody>
                    {isMarketOrdersLoading ? (
                      <SkeletonMarketOrderRows />
                    ) : marketOrders.length > 0 ? (
                      marketOrders.map((order) => (
                        <tr
                          key={order.id}
                          className="transition hover:bg-surface"
                        >
                          <td className="border-b border-border px-3 py-3">
                            <OrderSideBadge side={order.side} />
                          </td>
                          <td className="border-b border-border px-3 py-3 font-mono text-text">
                            <div>{formatShares(order.shares)}</div>
                            <div className="text-xs text-secondary">
                              {formatShares(order.remaining_shares)} remaining
                            </div>
                          </td>
                          <td className="border-b border-border px-3 py-3 font-mono text-text">
                            {formatCurrency(order.price)}
                          </td>
                          <td className="border-b border-border px-3 py-3">
                            <OrderStatusBadge status={order.status} />
                          </td>
                          <td className="border-b border-border px-3 py-3 font-mono text-xs text-secondary">
                            {order.id}
                          </td>
                          <td className="border-b border-border px-3 py-3 text-secondary">
                            {formatDateTime(order.created_at)}
                          </td>
                        </tr>
                      ))
                    ) : (
                      <tr>
                        <td
                          colSpan={6}
                          className="px-3 py-6 text-center text-secondary"
                        >
                          No orders found for this market.
                        </td>
                      </tr>
                    )}
                  </tbody>
                </table>
              </div>

              <div className="flex flex-col gap-2 sm:flex-row sm:justify-end">
                <button
                  type="button"
                  disabled={
                    marketOrderQuery.skip === 0 || isMarketOrdersLoading
                  }
                  onClick={handlePreviousMarketOrders}
                  className="h-10 rounded-lg border border-border bg-surface px-4 text-sm font-semibold text-text transition hover:border-accent disabled:opacity-40"
                >
                  Previous
                </button>
                <button
                  type="button"
                  disabled={
                    marketOrders.length < marketOrderQuery.limit ||
                    isMarketOrdersLoading
                  }
                  onClick={handleNextMarketOrders}
                  className="h-10 rounded-lg border border-border bg-surface px-4 text-sm font-semibold text-text transition hover:border-accent disabled:opacity-40"
                >
                  Next
                </button>
              </div>
            </div>
          ) : null}
        </div>
      ) : null}
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

function SelectField({
  id,
  label,
  value,
  options,
  onChange,
}: {
  id: string;
  label: string;
  value: string;
  options: Array<{ label: string; value: string }>;
  onChange: (value: string) => void;
}) {
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

function OrderSideBadge({ side }: { side: Order["side"] }) {
  const isBuy = side === "BUY";

  return (
    <span
      className={`inline-flex rounded-full border px-2.5 py-1 text-xs font-semibold ${
        isBuy
          ? "border-buy/30 bg-buy/15 text-buy"
          : "border-sell/30 bg-sell/15 text-sell"
      }`}
    >
      {isBuy ? "Buy" : "Sell"}
    </span>
  );
}

function OrderStatusBadge({ status }: { status: Order["status"] }) {
  const classes = {
    PENDING: "border-accent/30 bg-accent/15 text-accent",
    FILLED: "border-buy/30 bg-buy/15 text-buy",
    CANCELLED: "border-border bg-card text-secondary",
    PARTIAL: "border-accent/30 bg-accent/15 text-accent",
    EXPIRED: "border-border bg-card text-secondary",
  } satisfies Record<Order["status"], string>;

  return (
    <span
      className={`inline-flex rounded-full border px-2.5 py-1 text-xs font-semibold ${classes[status]}`}
    >
      {status}
    </span>
  );
}

function formatCurrency(value: string | number) {
  const numericValue = typeof value === "number" ? value : Number(value);

  if (!Number.isFinite(numericValue)) {
    return "--";
  }

  return new Intl.NumberFormat("en", {
    style: "currency",
    currency: "USD",
    maximumFractionDigits: 2,
  }).format(numericValue);
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

function MarketDetailsSkeleton() {
  return (
    <section className="mx-auto w-full max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
      <div className="grid gap-6 lg:grid-cols-3">
        <div className="space-y-6 lg:col-span-2">
          <div className="rounded-2xl border border-border bg-surface p-6">
            <div className="mb-6 flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
              <div className="space-y-3">
                <div className="h-4 w-28 rounded bg-card animate-pulse" />
                <div className="h-8 w-3/4 rounded bg-card animate-pulse" />
                <div className="h-4 w-full max-w-3xl rounded bg-card animate-pulse" />
                <div className="h-4 w-5/6 rounded bg-card animate-pulse" />
              </div>
              <div className="flex flex-wrap items-center gap-2">
                <div className="h-7 w-24 rounded-full bg-card animate-pulse" />
                <div className="h-7 w-20 rounded-full bg-card animate-pulse" />
              </div>
            </div>

            <div className="h-[280px] rounded-xl border border-border bg-card animate-pulse sm:h-[320px]" />
          </div>

          <div className="rounded-2xl border border-border bg-card p-6">
            <div className="mb-4 h-6 w-36 rounded bg-surface animate-pulse" />
            <div className="overflow-hidden rounded-xl border border-border bg-surface">
              <div className="grid grid-cols-2 gap-4 border-b border-border px-4 py-3 sm:grid-cols-4">
                <div className="h-3 w-16 rounded bg-card animate-pulse" />
                <div className="h-3 w-16 rounded bg-card animate-pulse" />
                <div className="h-3 w-16 rounded bg-card animate-pulse" />
                <div className="h-3 w-16 rounded bg-card animate-pulse" />
              </div>
              <div className="space-y-3 p-4">
                {Array.from({ length: 6 }).map((_, index) => (
                  <div
                    key={index}
                    className="grid grid-cols-2 gap-4 sm:grid-cols-4"
                  >
                    <div className="h-4 w-20 rounded bg-card animate-pulse" />
                    <div className="h-4 w-24 rounded bg-card animate-pulse" />
                    <div className="h-4 w-24 rounded bg-card animate-pulse" />
                    <div className="h-4 w-24 rounded bg-card animate-pulse" />
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>

        <div className="space-y-6 lg:col-span-1">
          <div className="rounded-2xl border border-border bg-surface p-6">
            <div className="h-6 w-40 rounded bg-card animate-pulse" />

            <div className="mt-6 space-y-4">
              <div className="h-11 rounded-lg bg-card animate-pulse" />
              <div className="h-11 rounded-lg bg-card animate-pulse" />
              <div className="h-11 rounded-lg bg-card animate-pulse" />
            </div>

            <div className="mt-6 space-y-3 border-t border-border pt-4">
              <div className="space-y-2">
                <div className="h-3 w-28 rounded bg-card animate-pulse" />
                <div className="h-4 w-32 rounded bg-card animate-pulse" />
              </div>
              <div className="space-y-2">
                <div className="h-3 w-24 rounded bg-card animate-pulse" />
                <div className="h-4 w-28 rounded bg-card animate-pulse" />
              </div>
              <div className="space-y-2">
                <div className="h-3 w-16 rounded bg-card animate-pulse" />
                <div className="h-4 w-24 rounded bg-card animate-pulse" />
              </div>
              <div className="space-y-2">
                <div className="h-3 w-16 rounded bg-card animate-pulse" />
                <div className="h-4 w-24 rounded bg-card animate-pulse" />
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}

function SkeletonMarketOrderRows() {
  return (
    <>
      {Array.from({ length: 5 }).map((_, index) => (
        <tr key={index} className="animate-pulse">
          <td className="border-b border-border px-3 py-3">
            <div className="h-6 w-14 rounded-full bg-surface" />
          </td>
          <td className="border-b border-border px-3 py-3">
            <div className="space-y-2">
              <div className="h-4 w-20 rounded bg-surface" />
              <div className="h-3 w-28 rounded bg-surface" />
            </div>
          </td>
          <td className="border-b border-border px-3 py-3">
            <div className="h-4 w-16 rounded bg-surface" />
          </td>
          <td className="border-b border-border px-3 py-3">
            <div className="h-6 w-20 rounded-full bg-surface" />
          </td>
          <td className="border-b border-border px-3 py-3">
            <div className="h-3 w-28 rounded bg-surface" />
          </td>
          <td className="border-b border-border px-3 py-3">
            <div className="h-3 w-24 rounded bg-surface" />
          </td>
        </tr>
      ))}
    </>
  );
}
