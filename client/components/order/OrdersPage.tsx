"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useEffect, useRef, useState } from "react";
import { useAuth } from "@/components/auth/AuthProvider";
import { ApiError } from "@/lib/api/http";
import { getOrders } from "@/lib/order/order-api";
import type {
  Order,
  OrdersQuery,
  OrderSide,
  OrderStatus,
} from "@/lib/order/types";

const DEFAULT_ORDER_QUERY: Required<OrdersQuery> = {
  market_id: "",
  outcome_id: "",
  order_by: "DESC",
  order_field: "created_at",
  side: "",
  status: "",
  order_type: "",
  limit: 10,
  skip: 0,
  before: "",
  after: "",
};

const ORDER_ROW_HEIGHT = 78;
const ORDER_VIEWPORT_HEIGHT = 600;
const ORDER_OVERSCAN = 5;

export function OrdersPage() {
  const router = useRouter();
  const { isAuthenticated, isLoading } = useAuth();
  const [orders, setOrders] = useState<Order[]>([]);
  const [query, setQuery] = useState(DEFAULT_ORDER_QUERY);
  const [isOrdersLoading, setIsOrdersLoading] = useState(true);
  const [hasMoreOrders, setHasMoreOrders] = useState(true);
  const [scrollTop, setScrollTop] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const ordersViewportRef = useRef<HTMLDivElement>(null);
  const loadMoreRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (isLoading) {
      return;
    }

    if (!isAuthenticated) {
      // eslint-disable-next-line react-hooks/set-state-in-effect
      setOrders([]);
      setError(null);
      setIsOrdersLoading(false);
      setHasMoreOrders(false);
      return;
    }

    let isCurrent = true;
    setIsOrdersLoading(true);
    setError(null);

    getOrders(query)
      .then((response) => {
        if (isCurrent) {
          setOrders((currentOrders) =>
            query.skip === 0 ? response : [...currentOrders, ...response]
          );
          setHasMoreOrders(response.length === query.limit);
        }
      })
      .catch((caughtError: unknown) => {
        if (isCurrent) {
          setError(getOrdersError(caughtError, "Unable to load orders."));
        }
      })
      .finally(() => {
        if (isCurrent) {
          setIsOrdersLoading(false);
        }
      });

    return () => {
      isCurrent = false;
    };
  }, [isAuthenticated, isLoading, query]);

  function handleFilterChange(key: keyof Required<OrdersQuery>, value: string) {
    setIsOrdersLoading(true);
    setOrders([]);
    setScrollTop(0);
    ordersViewportRef.current?.scrollTo({ top: 0 });
    setQuery((currentQuery) => ({
      ...currentQuery,
      [key]: key === "limit" || key === "skip" ? Number(value) : value,
      skip: 0,
    }));
  }

  useEffect(() => {
    const viewport = ordersViewportRef.current;
    const loadMore = loadMoreRef.current;

    if (!viewport || !loadMore || !isAuthenticated || !hasMoreOrders) {
      return;
    }

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting && !isOrdersLoading) {
          setIsOrdersLoading(true);
          setQuery((currentQuery) => ({
            ...currentQuery,
            skip: currentQuery.skip + currentQuery.limit,
          }));
        }
      },
      { root: viewport, rootMargin: "240px" }
    );

    observer.observe(loadMore);
    return () => observer.disconnect();
  }, [hasMoreOrders, isAuthenticated, isOrdersLoading]);

  if (isLoading) {
    return (
      <OrdersShell
        title="Loading orders"
        description="Checking your session..."
      />
    );
  }

  if (!isAuthenticated) {
    return (
      <OrdersShell
        title="Orders"
        description="Sign in to view your recent orders."
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
            <p className="text-sm text-secondary">Trading activity</p>
            <h1 className="mt-1 text-2xl font-bold text-text sm:text-3xl">
              Review your orders
            </h1>
          </div>
          <div className="grid gap-2 sm:grid-cols-3 lg:w-[480px]">
            <SelectField
              id="order-side"
              label="Side"
              value={query.side}
              onChange={(value) => handleFilterChange("side", value)}
              options={[
                { label: "All", value: "" },
                { label: "Buy", value: "BUY" },
                { label: "Sell", value: "SELL" },
              ]}
            />
            <SelectField
              id="order-status"
              label="Status"
              value={query.status}
              onChange={(value) => handleFilterChange("status", value)}
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
              id="order-type"
              label="Order type"
              value={query.order_type}
              onChange={(value) => handleFilterChange("order_type", value)}
              options={[
                { label: "All", value: "" },
                { label: "Limit", value: "LIMIT" },
                { label: "Market", value: "MARKET" },
              ]}
            />
          </div>
        </div>

        {error ? (
          <div className="mt-6 rounded-xl border border-accent/40 bg-card p-4 text-sm text-secondary">
            {error}
          </div>
        ) : null}

        <div
          ref={ordersViewportRef}
          onScroll={(event) => setScrollTop(event.currentTarget.scrollTop)}
          className="mt-6 max-h-[min(65vh,600px)] overflow-auto rounded-xl border border-border bg-card [scrollbar-width:none] [&::-webkit-scrollbar]:hidden"
        >
          <table className="w-full min-w-[760px] border-collapse text-left text-sm">
            <thead className="sticky top-0 z-10 bg-surface text-xs uppercase text-secondary">
              <tr>
                <th className="border-b border-border px-3 py-3 font-medium">
                  Order type
                </th>
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
                  Created
                </th>
              </tr>
            </thead>
            <tbody style={{ height: orders.length * ORDER_ROW_HEIGHT }}>
              {isOrdersLoading && orders.length === 0 ? (
                <SkeletonOrderRows />
              ) : orders.length > 0 ? (
                <VirtualizedOrderRows
                  orders={orders}
                  router={router}
                  scrollTop={scrollTop}
                />
              ) : (
                <tr>
                  <td
                    colSpan={6}
                    className="px-3 py-6 text-center text-secondary"
                  >
                    No orders found.
                  </td>
                </tr>
              )}
            </tbody>
          </table>
          <div
            ref={loadMoreRef}
            className="flex h-12 items-center justify-center"
          >
            {isOrdersLoading && orders.length > 0 ? (
              <span className="text-xs text-secondary">
                Loading more orders...
              </span>
            ) : !hasMoreOrders && orders.length > 0 ? (
              <span className="text-xs text-secondary">End of orders</span>
            ) : null}
          </div>
        </div>
      </div>
    </section>
  );
}

function OrdersShell({
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
        <p className="text-sm text-secondary">Orders</p>
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

function VirtualizedOrderRows({
  orders,
  router,
  scrollTop,
}: {
  orders: Order[];
  router: ReturnType<typeof useRouter>;
  scrollTop: number;
}) {
  const firstVisibleIndex = Math.floor(scrollTop / ORDER_ROW_HEIGHT);
  const firstRenderedIndex = Math.max(0, firstVisibleIndex - ORDER_OVERSCAN);
  const lastRenderedIndex = Math.min(
    orders.length,
    Math.ceil((scrollTop + ORDER_VIEWPORT_HEIGHT) / ORDER_ROW_HEIGHT) +
      ORDER_OVERSCAN
  );
  const visibleOrders = orders.slice(firstRenderedIndex, lastRenderedIndex);

  return (
    <>
      <tr
        aria-hidden="true"
        style={{ height: firstRenderedIndex * ORDER_ROW_HEIGHT }}
      >
        <td colSpan={6} />
      </tr>
      {visibleOrders.map((order) => (
        <tr
          key={order.id}
          tabIndex={0}
          role="button"
          style={{ height: ORDER_ROW_HEIGHT }}
          onClick={() => router.push(`/orders/${order.id}`)}
          onKeyDown={(event) => {
            if (event.key === "Enter" || event.key === " ") {
              event.preventDefault();
              router.push(`/orders/${order.id}`);
            }
          }}
          className="cursor-pointer transition hover:bg-surface focus:bg-surface focus:outline-none focus:ring-2 focus:ring-inset focus:ring-accent"
        >
          <td className="border-b border-border px-3 py-3 text-text">
            {order.order_type}
          </td>
          <td className="border-b border-border px-3 py-3">
            <OrderSideBadge side={order.side} />
          </td>
          <td className="border-b border-border px-3 py-3 font-mono text-text">
            <div>{formatShares(order.shares)}</div>
            {!isMarketBuyOrder(order) ? (
              <div className="mt-1 text-xs text-secondary">
                {formatShares(order.remaining_shares)} remaining
              </div>
            ) : null}
          </td>
          <td className="border-b border-border px-3 py-3 font-mono text-text">
            {isMarketBuyOrder(order) ? (
              <>
                <div>{formatCurrency(order.quote_amount)}</div>
                <div className="mt-1 text-xs text-secondary">
                  {formatCurrency(order.remaining_quote)} remaining
                </div>
              </>
            ) : isMarketSellOrder(order) ? (
              <div>{formatCurrency(order.average_price)} average</div>
            ) : (
              <>
                <div>{formatCurrency(order.price)}</div>
                <div className="mt-1 text-xs text-secondary">
                  {formatCurrency(order.average_price)} average
                </div>
              </>
            )}
          </td>
          <td className="border-b border-border px-3 py-3">
            <OrderStatusBadge status={order.status} />
          </td>
          <td className="border-b border-border px-3 py-3 text-secondary">
            {formatDateTime(order.created_at) ? (
              <>
                <div className="text-text">
                  {formatDateTime(order.created_at)?.time}
                </div>
                <div className="mt-1 text-xs text-secondary">
                  {formatDateTime(order.created_at)?.date}
                </div>
              </>
            ) : (
              "Unavailable"
            )}
          </td>
        </tr>
      ))}
      <tr
        aria-hidden="true"
        style={{
          height: (orders.length - lastRenderedIndex) * ORDER_ROW_HEIGHT,
        }}
      >
        <td colSpan={6} />
      </tr>
    </>
  );
}

function OrderSideBadge({ side }: { side: string }) {
  const normalizedSide = side.toUpperCase() as OrderSide;
  const className =
    normalizedSide === "BUY"
      ? "border-buy/30 bg-buy/15 text-buy"
      : normalizedSide === "SELL"
        ? "border-sell/30 bg-sell/15 text-sell"
        : "border-border bg-surface text-secondary";

  return (
    <span
      className={`rounded-full border px-2.5 py-1 text-xs font-semibold ${className}`}
    >
      {side}
    </span>
  );
}

function OrderStatusBadge({ status }: { status: string }) {
  const normalizedStatus = status.toUpperCase() as OrderStatus;
  const className =
    normalizedStatus === "FILLED"
      ? "border-buy/30 bg-buy/15 text-buy"
      : normalizedStatus === "CANCELLED" || normalizedStatus === "EXPIRED"
        ? "border-sell/30 bg-sell/15 text-sell"
        : normalizedStatus === "PARTIAL"
          ? "border-accent/30 bg-accent/15 text-accent"
          : "border-border bg-surface text-secondary";

  return (
    <span
      className={`rounded-full border px-2.5 py-1 text-xs font-semibold ${className}`}
    >
      {status.replace("_", " ")}
    </span>
  );
}

function isMarketBuyOrder(order: Order) {
  return order.order_type === "MARKET" && order.side === "BUY";
}

function isMarketSellOrder(order: Order) {
  return order.order_type === "MARKET" && order.side === "SELL";
}

function formatCurrency(value: string | number | null | undefined) {
  if (value === null || value === undefined || Number.isNaN(Number(value))) {
    return "--";
  }

  return new Intl.NumberFormat("en-US", {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(Number(value));
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
    return null;
  }

  const date = new Date(value);

  if (Number.isNaN(date.getTime())) {
    return null;
  }

  return {
    time: new Intl.DateTimeFormat("en-US", {
      timeStyle: "short",
    }).format(date),
    date: new Intl.DateTimeFormat("en-US", {
      dateStyle: "medium",
    }).format(date),
  };
}

function getOrdersError(caughtError: unknown, fallback: string) {
  if (caughtError instanceof ApiError) {
    return caughtError.message;
  }

  if (caughtError instanceof Error) {
    return caughtError.message;
  }

  return fallback;
}

function SkeletonOrderRows() {
  return (
    <>
      {Array.from({ length: 5 }).map((_, index) => (
        <tr key={index} className="animate-pulse">
          <td className="border-b border-border px-3 py-3">
            <div className="h-4 w-20 rounded bg-surface" />
          </td>
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
            <div className="space-y-2">
              <div className="h-4 w-16 rounded bg-surface" />
              <div className="h-3 w-28 rounded bg-surface" />
            </div>
          </td>
          <td className="border-b border-border px-3 py-3">
            <div className="h-4 w-20 rounded bg-surface" />
          </td>
          <td className="border-b border-border px-3 py-3">
            <div className="h-6 w-20 rounded-full bg-surface" />
          </td>
          <td className="border-b border-border px-3 py-3">
            <div className="h-3 w-24 rounded bg-surface" />
          </td>
        </tr>
      ))}
    </>
  );
}
