"use client";

import Link from "next/link";
import { useEffect, useState } from "react";
import { useAuth } from "@/components/auth/AuthProvider";
import { ApiError } from "@/lib/api/http";
import { getOrder } from "@/lib/order/order-api";
import type { OrderDetail } from "@/lib/order/types";

export function OrderDetailsPage({ orderId }: { orderId: string }) {
  const { isAuthenticated, isLoading } = useAuth();
  const [order, setOrder] = useState<OrderDetail | null>(null);
  const [isOrderLoading, setIsOrderLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (isLoading) {
      return;
    }

    if (!isAuthenticated) {
      // eslint-disable-next-line react-hooks/set-state-in-effect
      setIsOrderLoading(false);
      return;
    }

    let isCurrent = true;
    setIsOrderLoading(true);
    setError(null);

    getOrder(orderId)
      .then((response) => {
        if (isCurrent) {
          setOrder(response);
        }
      })
      .catch((caughtError: unknown) => {
        if (isCurrent) {
          setError(getOrderError(caughtError));
        }
      })
      .finally(() => {
        if (isCurrent) {
          setIsOrderLoading(false);
        }
      });

    return () => {
      isCurrent = false;
    };
  }, [isAuthenticated, isLoading, orderId]);

  if (isLoading || isOrderLoading) {
    return <OrderDetailsShell isLoading />;
  }

  if (!isAuthenticated) {
    return (
      <OrderDetailsShell
        title="Order details"
        description="Sign in to view this order."
        actionHref="/login"
        actionLabel="Log in"
      />
    );
  }

  if (error || !order) {
    return (
      <OrderDetailsShell
        title="Unable to load order"
        description={error ?? "This order could not be found."}
        actionHref="/orders"
        actionLabel="Back to orders"
      />
    );
  }

  const isLimitOrder = order.order_type === "LIMIT";
  const isMarketBuy = order.order_type === "MARKET" && order.side === "BUY";
  const isMarketSell = order.order_type === "MARKET" && order.side === "SELL";

  return (
    <section className="mx-auto w-full max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
      <div className="rounded-2xl border border-border bg-surface p-4 sm:p-6">
        <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
          <div>
            <Link
              href="/orders"
              className="text-sm font-medium text-secondary transition hover:text-text"
            >
              Back to orders
            </Link>
            <h1
              className={`mt-1 break-all text-2xl font-bold sm:text-3xl ${getOrderStatusClassName(order.status)}`}
            >
              {order.status} Order
            </h1>
          </div>
          <Link
            href={`/markets/${order.market_id}`}
            className="inline-flex h-10 items-center justify-center rounded-lg border border-accent bg-accent px-3 text-sm font-semibold text-text transition hover:brightness-110"
          >
            OPEN MARKET
          </Link>
        </div>

        <div className="mt-6 grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
          <OrderIdValue orderId={order.id} />
          <OrderDetailValue label="Order type" value={order.order_type} />
          <OrderDetailValue label="Side" value={order.side} />
          <OrderDetailValue label="Shares" value={formatNumber(order.shares)} />
          {!isMarketBuy ? (
            <OrderDetailValue
              label="Remaining shares"
              value={formatNumber(order.remaining_shares)}
            />
          ) : null}
          {!isMarketBuy && !isMarketSell ? (
            <OrderDetailValue label="Price" value={formatNumber(order.price)} />
          ) : null}
          <OrderDetailValue
            label="Average price"
            value={formatNumber(order.average_price)}
          />
          {!isLimitOrder && isMarketBuy ? (
            <>
              <OrderDetailValue
                label="Quote amount"
                value={formatNumber(order.quote_amount)}
              />
              <OrderDetailValue
                label="Remaining quote"
                value={formatNumber(order.remaining_quote)}
              />
            </>
          ) : null}
          {isLimitOrder ? (
            <OrderDetailValue
              label="Expires"
              value={formatDateTime(order.expires_at)}
            />
          ) : null}
          <OrderDetailValue
            label="Created"
            value={formatDateTime(order.created_at)}
          />
        </div>

        <div className="mt-8">
          <div className="mt-4 overflow-x-auto rounded-xl border border-border bg-card">
            <table className="w-full min-w-[760px] border-collapse text-left text-sm">
              <thead className="bg-surface text-xs uppercase text-secondary">
                <tr>
                  <th className="border-b border-border px-3 py-3 font-medium">
                    {order.side === "BUY" ? "Bought" : "Sold"} Shares
                  </th>
                  <th className="border-b border-border px-3 py-3 font-medium">
                    For Price
                  </th>
                  <th className="border-b border-border px-3 py-3 font-medium">
                    {order.side === "BUY" ? "Bought" : "Sold"} At
                  </th>
                </tr>
              </thead>
              <tbody>
                {order.trade.length > 0 ? (
                  order.trade.map((trade) => (
                    <tr key={trade.id}>
                      <td className="border-b border-border px-3 py-3 font-mono text-text">
                        {formatNumber(trade.shares)}
                      </td>
                      <td className="border-b border-border px-3 py-3 font-mono text-text">
                        {formatNumber(trade.price)}
                      </td>
                      <td className="border-b border-border px-3 py-3 text-secondary">
                        {formatDateTime(trade.created_at)}
                      </td>
                    </tr>
                  ))
                ) : (
                  <tr>
                    <td
                      colSpan={3}
                      className="px-3 py-6 text-center text-secondary"
                    >
                      No trades for this order.
                    </td>
                  </tr>
                )}
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </section>
  );
}

function OrderDetailsShell({
  title = "Loading order",
  description = "Loading order details...",
  actionHref,
  actionLabel,
  isLoading = false,
}: {
  title?: string;
  description?: string;
  actionHref?: string;
  actionLabel?: string;
  isLoading?: boolean;
}) {
  return (
    <section className="mx-auto w-full max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
      <div className="rounded-2xl border border-border bg-surface p-4 sm:p-6">
        {isLoading ? (
          <>
            <div className="h-4 w-28 rounded bg-card animate-pulse" />
            <div className="mt-3 h-8 w-2/3 rounded bg-card animate-pulse" />
            <div className="mt-2 h-4 w-48 rounded bg-card animate-pulse" />
            <div className="mt-6 grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
              {Array.from({ length: 12 }).map((_, index) => (
                <div
                  key={index}
                  className="rounded-lg border border-border bg-card p-3"
                >
                  <div className="h-3 w-20 rounded bg-surface animate-pulse" />
                  <div className="mt-2 h-5 w-28 rounded bg-surface animate-pulse" />
                </div>
              ))}
            </div>
          </>
        ) : (
          <>
            <p className="text-sm text-secondary">Order details</p>
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
          </>
        )}
      </div>
    </section>
  );
}

function OrderDetailValue({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-lg border border-border bg-card p-3">
      <dt className="text-xs uppercase text-secondary">{label}</dt>
      <dd className="mt-1 break-all font-mono text-sm text-text">{value}</dd>
    </div>
  );
}

function OrderIdValue({ orderId }: { orderId: string }) {
  const [isCopied, setIsCopied] = useState(false);

  async function handleCopy() {
    await navigator.clipboard.writeText(orderId);
    setIsCopied(true);
    window.setTimeout(() => setIsCopied(false), 1500);
  }

  return (
    <div className="rounded-lg border border-border bg-card p-3 sm:col-span-2 lg:col-span-2">
      <div className="flex items-center justify-between gap-3">
        <span className="text-xs uppercase text-secondary">Order ID</span>
        <button
          type="button"
          onClick={handleCopy}
          className="rounded-md border border-border bg-surface px-2 py-1 text-xs font-semibold text-secondary transition hover:border-accent hover:text-text"
        >
          {isCopied ? "Copied" : "Copy"}
        </button>
      </div>
      <p className="mt-1 break-all font-mono text-sm text-text">{orderId}</p>
    </div>
  );
}

function getOrderStatusClassName(status: OrderDetail["status"]) {
  return status === "FILLED"
    ? "text-buy"
    : status === "CANCELLED" || status === "EXPIRED"
      ? "text-sell"
      : status === "PARTIAL" || status === "PENDING"
        ? "text-accent"
        : "text-secondary";
}

function formatNumber(value: string | number | null | undefined) {
  if (value === null || value === undefined || Number.isNaN(Number(value))) {
    return "--";
  }

  return new Intl.NumberFormat("en-US", {
    minimumFractionDigits: 2,
    maximumFractionDigits: 8,
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

function getOrderError(caughtError: unknown) {
  if (caughtError instanceof ApiError || caughtError instanceof Error) {
    return caughtError.message;
  }

  return "Unable to load order details.";
}
