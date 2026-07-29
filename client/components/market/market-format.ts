import type { MarketStatus, OutcomeSnapshot } from "@/lib/market/types";

export function formatDateTime(value: string | number) {
  const date = typeof value === "number" ? new Date(value) : new Date(value);

  if (Number.isNaN(date.getTime())) {
    return "Unknown";
  }

  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
    year: "numeric",
    hour: "numeric",
    minute: "2-digit",
  }).format(date);
}

export function formatProbability(value: string | number | null | undefined) {
  const numericValue = typeof value === "number" ? value : Number(value);

  if (!Number.isFinite(numericValue)) {
    return "--";
  }

  return `${Math.round(numericValue * 100)}`;
}

export function formatShares(value: string | number) {
  const numericValue = typeof value === "number" ? value : Number(value);

  if (!Number.isFinite(numericValue)) {
    return "--";
  }

  return new Intl.NumberFormat("en", {
    maximumFractionDigits: 2,
  }).format(numericValue);
}

export function getBestSellPrice(snapshot: OutcomeSnapshot | undefined) {
  if (!snapshot || snapshot.sell.length === 0) {
    return null;
  }

  return Math.min(...snapshot.sell.map((level) => Number(level.price)).filter(Number.isFinite));
}

export function getStatusClassName(status: MarketStatus) {
  if (status === "ACTIVE") {
    return "border-buy/30 bg-buy/15 text-buy";
  }

  if (status === "CLOSED" || status === "RESOLVED" || status === "CANCELLED") {
    return "border-border bg-card text-secondary";
  }

  return "border-accent/30 bg-accent/15 text-accent";
}
