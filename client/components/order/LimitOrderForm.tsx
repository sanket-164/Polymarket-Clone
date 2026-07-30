"use client";

import { useState } from "react";
import { placeOrder } from "@/lib/order/order-api";
import type { OrderSide } from "@/lib/order/types";
import type { Outcome } from "@/lib/market/types";

interface LimitOrderFormProps {
  marketId: string;
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
  firstOutcome,
  secondOutcome,
  currentPrices,
  onSuccess,
}: LimitOrderFormProps) {
  const [formState, setFormState] = useState<FormState>({
    side: "BUY",
    outcomeId: firstOutcome.id,
    shares: 0,
    price: "",
  });
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  // Get real-time prices from WebSocket, fallback to initial market price
  const liveFirstPrice =
    currentPrices[firstOutcome.id] ?? firstOutcome.current_price;
  const liveSecondPrice =
    currentPrices[secondOutcome.id] ?? secondOutcome.current_price;

  const selectedLivePrice =
    formState.outcomeId === firstOutcome.id ? liveFirstPrice : liveSecondPrice;

  // Calculate total cost and potential win
  const priceNum = parseFloat(formState.price) || 0;
  const totalCost = formState.shares * priceNum;
  const potentialWin =
    formState.side === "BUY"
      ? formState.shares * (1 - priceNum)
      : formState.shares * priceNum;

  const handleSideChange = (side: OrderSide) => {
    setFormState((prev) => ({ ...prev, side }));
    setError(null);
    setSuccessMessage(null);
  };

  const handleOutcomeChange = (outcomeId: string) => {
    const newLivePrice =
      outcomeId === firstOutcome.id ? liveFirstPrice : liveSecondPrice;

    setFormState((prev) => {
      // Auto-set the price to the market price ONLY when switching outcomes
      // or if the price field is currently empty. This prevents overwriting
      // manual edits if the user accidentally clicks the same outcome again.
      if (prev.outcomeId !== outcomeId || prev.price === "") {
        return {
          ...prev,
          outcomeId,
          price: Number(newLivePrice).toFixed(2),
        };
      }

      // If clicking the same outcome and a price already exists, keep the existing price
      return {
        ...prev,
        outcomeId,
      };
    });

    setError(null);
    setSuccessMessage(null);
  };

  const handleSharesChange = (delta: number) => {
    setFormState((prev) => {
      const newShares = Math.max(0, prev.shares + delta);
      return { ...prev, shares: newShares };
    });
  };

  const handleSubmit = async () => {
    if (formState.shares <= 0) {
      setError("Shares must be greater than 0");
      return;
    }
    if (priceNum <= 0 || priceNum > 1) {
      setError("Price must be between 0.01 and 1.00");
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
      });

      setSuccessMessage("Order placed successfully!");
      setFormState((prev) => ({ ...prev, shares: 0, price: "" }));
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
    <div className="rounded-2xl bg-surface p-6">
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
            Limit price
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
              // Allow empty string or valid floating point numbers while typing
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

      {/* Summary (No currency signs) */}
      <div className="space-y-2 mb-4 p-3 bg-card/30 rounded-lg">
        <div className="flex justify-between text-sm">
          <span className="text-secondary">Total</span>
          <span className="font-mono text-text">{totalCost.toFixed(2)}</span>
        </div>
        <div className="flex justify-between text-sm">
          <span className="text-secondary flex items-center gap-1">
            To win
            <svg
              className="w-3 h-3"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
          </span>
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
    </div>
  );
}
