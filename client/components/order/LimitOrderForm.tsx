"use client";

import Link from "next/link";
import { useEffect, useMemo, useRef, useState } from "react";
import { useAuth } from "@/components/auth/AuthProvider";
import { placeOrder } from "@/lib/order/order-api";
import type { OrderSide } from "@/lib/order/types";
import type { Outcome } from "@/lib/market/types";

/* ---------------------------------------------------------------------------
 * Expiration dropdown
 * ------------------------------------------------------------------------- */

type ExpirationKey = "never" | "5m" | "1h" | "12h" | "24h" | "eod" | "custom";

const EXPIRATION_OPTIONS: { key: ExpirationKey; label: string }[] = [
  { key: "never", label: "Never" },
  { key: "5m", label: "5m" },
  { key: "1h", label: "1h" },
  { key: "12h", label: "12h" },
  { key: "24h", label: "24h" },
  { key: "eod", label: "End of day" },
  { key: "custom", label: "Custom" },
];

const MINUTE_MS = 60_000;
const HOUR_MS = 3_600_000;

function computeExpiresAt(
  key: ExpirationKey,
  customValue: string
): string | null {
  const now = new Date();

  switch (key) {
    case "5m":
      return new Date(now.getTime() + 5 * MINUTE_MS).toISOString();
    case "1h":
      return new Date(now.getTime() + HOUR_MS).toISOString();
    case "12h":
      return new Date(now.getTime() + 12 * HOUR_MS).toISOString();
    case "24h":
      return new Date(now.getTime() + 24 * HOUR_MS).toISOString();
    case "eod": {
      const endOfDay = new Date(now);
      endOfDay.setHours(23, 59, 59, 999);
      return endOfDay.toISOString();
    }
    case "custom": {
      const parsed = new Date(customValue);
      return Number.isNaN(parsed.getTime()) ? null : parsed.toISOString();
    }
    default:
      return null; // "never"
  }
}

function toLocalInputValue(date: Date): string {
  const pad = (n: number) => String(n).padStart(2, "0");
  return (
    `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}` +
    `T${pad(date.getHours())}:${pad(date.getMinutes())}`
  );
}

interface ExpirationSelectProps {
  value: ExpirationKey;
  customValue: string;
  onChange: (key: ExpirationKey) => void;
  onCustomChange: (value: string) => void;
}

function ExpirationSelect({
  value,
  customValue,
  onChange,
  onCustomChange,
}: ExpirationSelectProps) {
  const [isOpen, setIsOpen] = useState(false);
  const rootRef = useRef<HTMLDivElement>(null);

  // Close on outside click or Escape
  useEffect(() => {
    if (!isOpen) return;

    const handlePointerDown = (event: PointerEvent) => {
      if (rootRef.current && !rootRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    };
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") setIsOpen(false);
    };

    document.addEventListener("pointerdown", handlePointerDown);
    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("pointerdown", handlePointerDown);
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [isOpen]);

  const selectedLabel =
    EXPIRATION_OPTIONS.find((option) => option.key === value)?.label ?? "Never";

  return (
    <div ref={rootRef} className="relative">
      {/* Trigger row: "Expires" left, value + chevron right */}
      <div className="flex items-center justify-between">
        <span className="text-sm text-secondary">Expires</span>

        <button
          type="button"
          onClick={() => setIsOpen((open) => !open)}
          aria-haspopup="listbox"
          aria-expanded={isOpen}
          className="flex items-center gap-1.5 text-sm font-medium text-accent transition hover:text-accent/80"
        >
          {selectedLabel}
          <svg
            viewBox="0 0 12 12"
            className={`h-3 w-3 transition-transform duration-200 ${
              isOpen ? "rotate-180" : ""
            }`}
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <path d="M2 4l4 4 4-4" />
          </svg>
        </button>
      </div>

      {/* Floating menu (overlays content below, like the screenshot) */}
      {isOpen && (
        <ul
          role="listbox"
          aria-label="Order expiration"
          className="absolute right-0 z-30 mt-2 w-44 rounded-xl border border-border bg-surface py-2 shadow-2xl shadow-background/60"
        >
          {EXPIRATION_OPTIONS.map((option) => (
            <li key={option.key}>
              <button
                type="button"
                role="option"
                aria-selected={option.key === value}
                onClick={() => {
                  onChange(option.key);
                  setIsOpen(false);
                }}
                className={`block w-full px-4 py-2.5 text-left text-sm transition ${
                  option.key === value
                    ? "bg-card font-medium text-accent"
                    : "text-text hover:bg-card"
                }`}
              >
                {option.label}
              </button>
            </li>
          ))}
        </ul>
      )}

      {/* Custom date/time picker */}
      {value === "custom" && (
        <div className="mt-3">
          <input
            type="datetime-local"
            value={customValue}
            min={toLocalInputValue(new Date())}
            onChange={(event) => onCustomChange(event.target.value)}
            className="w-full rounded-lg border border-border bg-card px-3 py-2 font-mono text-sm text-text focus:border-accent focus:outline-none"
          />
        </div>
      )}
    </div>
  );
}

/* ---------------------------------------------------------------------------
 * Limit order form
 * ------------------------------------------------------------------------- */

interface LimitOrderFormProps {
  marketId: string;
  marketCloseAt: string;
  firstOutcome: Outcome;
  secondOutcome: Outcome;
  currentPrices: Record<string, string>;
  onSuccess?: () => void;
}

interface FormState {
  side: OrderSide;
  outcomeId: string;
  shares: number;
  price: string;
}

const QUICK_SHARE_OPTIONS = [-100, -10, +10, +100, +200];

export function LimitOrderForm({
  marketId,
  marketCloseAt,
  firstOutcome,
  secondOutcome,
  currentPrices,
  onSuccess,
}: LimitOrderFormProps) {
  const { isAuthenticated, isLoading } = useAuth();
  const [formState, setFormState] = useState<FormState>({
    side: "BUY",
    outcomeId: firstOutcome.id,
    shares: 100,
    price: firstOutcome.current_price.slice(0, 4),
  });
  const [expiration, setExpiration] = useState<ExpirationKey>("never");
  const [customExpiration, setCustomExpiration] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  const [isLoginPromptOpen, setIsLoginPromptOpen] = useState(false);

  const expiresAt = useMemo(
    () => computeExpiresAt(expiration, customExpiration),
    [expiration, customExpiration]
  );
  const effectiveExpiresAt = expiresAt ?? marketCloseAt;

  // Get real-time prices from WebSocket, fallback to initial market price
  const liveFirstPrice =
    currentPrices[firstOutcome.id] ?? firstOutcome.current_price;
  const liveSecondPrice =
    currentPrices[secondOutcome.id] ?? secondOutcome.current_price;

  const selectedLivePrice =
    formState.outcomeId === firstOutcome.id ? liveFirstPrice : liveSecondPrice;

  const getDefaultPriceForOutcome = (outcomeId: string) => {
    const livePrice =
      outcomeId === firstOutcome.id ? liveFirstPrice : liveSecondPrice;

    return Number(livePrice).toFixed(2);
  };

  // Calculate total cost and potential win
  const priceNum = parseFloat(formState.price) || 0;
  const totalCost = formState.shares * priceNum;
  const potentialWin =
    formState.side === "BUY"
      ? formState.shares * (1 - priceNum)
      : formState.shares * priceNum;

  const handleSideChange = (side: OrderSide) => {
    setFormState((prev) => ({
      ...prev,
      side,
      price: getDefaultPriceForOutcome(prev.outcomeId),
    }));
    setError(null);
    setSuccessMessage(null);
  };

  const handleOutcomeChange = (outcomeId: string) => {
    const newLivePrice =
      outcomeId === firstOutcome.id ? liveFirstPrice : liveSecondPrice;

    setFormState((prev) => ({
      ...prev,
      outcomeId,
      price: Number(newLivePrice).toFixed(2),
    }));

    setError(null);
    setSuccessMessage(null);
  };

  const handleSharesChange = (delta: number) => {
    setFormState((prev) => {
      const newShares = Math.max(0, prev.shares + delta);
      return { ...prev, shares: newShares };
    });
  };

  const handleExpirationChange = (key: ExpirationKey) => {
    setExpiration(key);
    // Pre-fill the custom picker with now + 1h the first time it's opened
    if (key === "custom" && !customExpiration) {
      setCustomExpiration(toLocalInputValue(new Date(Date.now() + HOUR_MS)));
    }
  };

  const handleSubmit = async () => {
    if (isLoading) {
      return;
    }

    if (!isAuthenticated) {
      setIsLoginPromptOpen(true);
      return;
    }

    if (formState.shares <= 0) {
      setError("Shares must be greater than 0");
      return;
    }
    if (priceNum <= 0 || priceNum > 1) {
      setError("Price must be between 0.01 and 1.00");
      return;
    }
    if (expiration === "custom" && !expiresAt) {
      setError("Please pick an expiration date and time");
      return;
    }

    setIsSubmitting(true);
    setError(null);
    setSuccessMessage(null);

    try {
      await placeOrder({
        market_id: marketId,
        outcome_id: formState.outcomeId,
        shares: formState.shares,
        price: priceNum,
        side: formState.side,
        expires_at: effectiveExpiresAt,
      });

      setSuccessMessage("Order placed successfully!");
      setFormState((prev) => ({
        ...prev,
        shares: 100,
        price: getDefaultPriceForOutcome(prev.outcomeId),
      }));
      onSuccess?.();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to place order");
    } finally {
      setIsSubmitting(false);
    }
  };

  const getOutcomeButtonClass = (outcomeId: string) => {
    const isSelected = formState.outcomeId === outcomeId;

    if (!isSelected) {
      return "bg-card/50 text-secondary hover:bg-card hover:text-text";
    }

    return formState.side === "BUY"
      ? "bg-green-600 text-white hover:bg-green-700"
      : "bg-red-600 text-white hover:bg-red-700";
  };

  return (
    <div className="rounded-2xl p-6">
      <h2 className="text-lg font-semibold text-text mb-4">Place Order</h2>

      {/* Buy/Sell Toggle */}
      <div className="flex gap-2 mb-4">
        <button
          type="button"
          onClick={() => handleSideChange("BUY")}
          className={`flex-1 py-2 px-4 rounded-lg font-semibold transition ${
            formState.side === "BUY"
              ? "bg-green-600 text-white"
              : "bg-card text-secondary hover:bg-card/70"
          }`}
        >
          Buy
        </button>
        <button
          type="button"
          onClick={() => handleSideChange("SELL")}
          className={`flex-1 py-2 px-4 rounded-lg font-semibold transition ${
            formState.side === "SELL"
              ? "bg-red-600 text-white"
              : "bg-card text-secondary hover:bg-card/70"
          }`}
        >
          Sell
        </button>
      </div>

      {/* Yes/No Outcome Selection (Displays real-time WebSocket price) */}
      <div className="grid grid-cols-2 gap-2 mb-4">
        <button
          type="button"
          onClick={() => handleOutcomeChange(firstOutcome.id)}
          className={`py-3 px-4 rounded-lg font-semibold transition ${getOutcomeButtonClass(
            firstOutcome.id
          )}`}
        >
          <div className="text-sm">{firstOutcome.label}</div>
          <div className="text-xs opacity-80">
            {Number(liveFirstPrice).toFixed(2)}
          </div>
        </button>
        <button
          type="button"
          onClick={() => handleOutcomeChange(secondOutcome.id)}
          className={`py-3 px-4 rounded-lg font-semibold transition ${getOutcomeButtonClass(
            secondOutcome.id
          )}`}
        >
          <div className="text-sm">{secondOutcome.label}</div>
          <div className="text-xs opacity-80">
            {Number(liveSecondPrice).toFixed(2)}
          </div>
        </button>
      </div>

      {/* Limit Price Input */}
      <div className="mb-4">
        <div className="flex justify-between items-center mb-2">
          <label className="block text-sm font-medium text-secondary">
            Price
          </label>
          <button
            type="button"
            onClick={() =>
              setFormState((prev) => ({
                ...prev,
                price: Number(selectedLivePrice).toFixed(2),
              }))
            }
            className="text-xs text-accent hover:text-accent/80 transition font-medium"
          >
            Set to market ({Number(selectedLivePrice).toFixed(2)})
          </button>
        </div>
        <div className="flex items-center gap-2">
          <input
            type="text"
            inputMode="decimal"
            pattern="[0-9]*\.?[0-9]*"
            min="0"
            max="1"
            value={formState.price}
            onChange={(e) => {
              const val = e.target.value;
              if (val === "" || /^\d*\.?\d*$/.test(val)) {
                setFormState((prev) => ({ ...prev, price: val }));
              }
            }}
            className="flex-1 px-3 py-2 bg-card border border-border rounded-lg text-text text-right font-mono focus:outline-none focus:border-accent"
            placeholder="0.00"
          />
        </div>
      </div>

      {/* Shares Input */}
      <div className="mb-4">
        <label className="block text-sm font-medium text-secondary mb-2">
          Shares
        </label>
        <div className="flex items-center gap-2 mb-2">
          <input
            type="text"
            inputMode="decimal"
            pattern="[0-9]*"
            value={formState.shares || ""}
            onChange={(e) => {
              const num = Number(e.target.value);

              setFormState((prev) => ({
                ...prev,
                shares: Number.isNaN(num) ? 0 : Math.max(0, num),
              }));
            }}
            className="flex-1 px-3 py-2 bg-card border border-border rounded-lg text-text text-right font-mono focus:outline-none focus:border-accent"
            placeholder="0"
          />
        </div>
        <div className="flex flex-wrap gap-2">
          {QUICK_SHARE_OPTIONS.map((option) => (
            <button
              key={option}
              type="button"
              onClick={() => handleSharesChange(option)}
              className="px-3 py-1 text-xs rounded-lg bg-card hover:bg-card/70 text-secondary transition"
            >
              {option > 0 ? "+" : ""}
              {option}
            </button>
          ))}
        </div>
      </div>

      {/* Expiration Dropdown */}
      <div className="mb-4">
        <ExpirationSelect
          value={expiration}
          customValue={customExpiration}
          onChange={handleExpirationChange}
          onCustomChange={setCustomExpiration}
        />
      </div>

      {/* Summary (No currency signs) */}
      <div className="space-y-2 mb-4 p-3 bg-card/30 rounded-lg">
        <div className="flex justify-between text-sm">
          <span className="text-secondary">Total</span>
          <span className="font-mono text-text">{totalCost.toFixed(2)}</span>
        </div>
        <div className="flex justify-between text-sm">
          <span className="text-secondary flex items-center gap-1">Win</span>
          <span className="font-mono text-green-500">
            {potentialWin.toFixed(2)}
          </span>
        </div>
      </div>

      {/* Error/Success Messages */}
      {error && (
        <div className="mb-3 p-3 bg-red-500/10 border border-red-500/20 rounded-lg text-sm text-red-400">
          {error}
        </div>
      )}
      {successMessage && (
        <div className="mb-3 p-3 bg-green-500/10 border border-green-500/20 rounded-lg text-sm text-green-400">
          {successMessage}
        </div>
      )}

      {/* Trade Button */}
      <button
        type="button"
        onClick={handleSubmit}
        disabled={isSubmitting || formState.shares <= 0 || priceNum <= 0}
        className="w-full py-3 px-4 bg-blue-600 hover:bg-blue-700 disabled:bg-blue-600/50 disabled:cursor-not-allowed text-white font-semibold rounded-lg transition"
      >
        {isSubmitting ? "Placing..." : "Trade"}
      </button>

      {isLoginPromptOpen ? (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-background/80 px-4 backdrop-blur-sm"
          onClick={() => setIsLoginPromptOpen(false)}
        >
          <div
            className="w-full max-w-md rounded-2xl border border-border bg-surface p-6 shadow-2xl shadow-background/60"
            onClick={(event) => event.stopPropagation()}
          >
            <div className="flex items-start justify-between gap-4">
              <div>
                <p className="text-sm text-secondary">
                  Trading requires an account
                </p>
                <h3 className="mt-1 text-xl font-semibold text-text">
                  Log in to place this order
                </h3>
              </div>
              <button
                type="button"
                onClick={() => setIsLoginPromptOpen(false)}
                className="rounded-lg border border-border bg-card px-3 py-1 text-sm text-secondary transition hover:text-text"
              >
                Close
              </button>
            </div>

            <p className="mt-4 text-sm leading-6 text-secondary">
              Sign in to submit limit orders, track positions, and manage your
              trades.
            </p>

            <div className="mt-6 grid gap-3 sm:grid-cols-2">
              <Link
                href="/login"
                className="inline-flex h-11 items-center justify-center rounded-lg bg-accent px-4 text-sm font-semibold text-text transition hover:brightness-110"
              >
                Log in
              </Link>
              <Link
                href="/signup"
                className="inline-flex h-11 items-center justify-center rounded-lg border border-border bg-card px-4 text-sm font-semibold text-text transition hover:border-accent"
              >
                Sign up
              </Link>
            </div>
          </div>
        </div>
      ) : null}
    </div>
  );
}
