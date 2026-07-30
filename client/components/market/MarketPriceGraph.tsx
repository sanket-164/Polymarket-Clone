"use client";

import { useMemo } from "react";
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from "recharts";
import { formatProbability } from "@/components/market/market-format";
import type { MarketDetails } from "@/lib/market/types";

type PriceHistoryPoint = {
  time: string;
  first: number;
  second: number;
};

type MarketPriceGraphProps = {
  firstOutcome: MarketDetails["first_outcome"];
  secondOutcome: MarketDetails["second_outcome"];
  priceHistory: PriceHistoryPoint[];
};

const BUY_COLOR = "#10b981"; // emerald-500
const SELL_COLOR = "#ef4444"; // rose-500

export function MarketPriceGraph({
  firstOutcome,
  secondOutcome,
  priceHistory,
}: MarketPriceGraphProps) {
  // Keep only the last 20 trade points to prevent clutter and focus on recent activity
  const recentHistory = useMemo(() => priceHistory.slice(-20), [priceHistory]);

  const currentFirstPrice =
    recentHistory.length > 0
      ? recentHistory[recentHistory.length - 1].first
      : Number(firstOutcome.current_price);

  const currentSecondPrice =
    recentHistory.length > 0
      ? recentHistory[recentHistory.length - 1].second
      : Number(secondOutcome.current_price);

  // Ensure the chart data always has at least two points to span the full width
  const chartData = useMemo(() => {
    if (recentHistory.length === 0) {
      const firstPrice = Number(
        firstOutcome.start_price ?? firstOutcome.current_price
      );
      const secondPrice = Number(
        secondOutcome.start_price ?? secondOutcome.current_price
      );
      return [
        { time: "Start", first: firstPrice, second: secondPrice },
        { time: "Now", first: firstPrice, second: secondPrice },
      ];
    }

    // Edge case: if there's only 1 point in history, duplicate it to span the width
    if (recentHistory.length === 1) {
      return [
        { ...recentHistory[0], time: "Start" },
        { ...recentHistory[0], time: "Now" },
      ];
    }

    return recentHistory;
  }, [recentHistory, firstOutcome, secondOutcome]);

  return (
    <div className="rounded-2xl bg-surface">
      <div className="h-72 w-full rounded-xl border border-border bg-card">
        <ResponsiveContainer width="100%" height="100%">
          <LineChart
            data={chartData}
            margin={{ top: 0, right: 0, left: 0, bottom: 0 }}
          >
            <CartesianGrid
              strokeDasharray="3 3"
              stroke="hsl(var(--border))"
              vertical={false}
            />

            <XAxis
              dataKey="time"
              fontSize={12}
              tickLine={false}
              axisLine={false}
              tick={{ fill: "hsl(var(--secondary))" }}
              padding={{ left: 0, right: 0 }}
            />

            <YAxis
              domain={[0, 1]}
              tickFormatter={(value) => `${Math.round(value * 100)}%`}
              fontSize={12}
              tickLine={false}
              axisLine={false}
              width={30}
              tick={{ fill: "hsl(var(--secondary))" }}
            />

            <Tooltip
              contentStyle={{
                backgroundColor: "hsl(var(--card))",
                borderColor: "hsl(var(--border))",
                borderRadius: "8px",
                color: "hsl(var(--text))",
              }}
              itemStyle={{ color: "hsl(var(--text))" }}
              labelStyle={{
                color: "hsl(var(--secondary))",
              }}
              formatter={(value, name) => {
                const numericValue =
                  typeof value === "number" ? value : Number(value);

                if (Number.isNaN(numericValue)) {
                  return ["-", ""];
                }

                return [
                  `${(numericValue * 100).toFixed(1)}%`,
                  name === "first" ? firstOutcome.label : secondOutcome.label,
                ];
              }}
              labelFormatter={(label) => `Time: ${label}`}
            />

            <Line
              type="monotone"
              dataKey="first"
              stroke={BUY_COLOR}
              strokeWidth={2}
              dot={false}
              isAnimationActive={false}
              activeDot={{
                r: 5,
                fill: BUY_COLOR,
                stroke: "hsl(var(--card))",
                strokeWidth: 2,
              }}
            />

            <Line
              type="monotone"
              dataKey="second"
              stroke={SELL_COLOR}
              strokeWidth={2}
              dot={false}
              isAnimationActive={false}
              activeDot={{
                r: 5,
                fill: SELL_COLOR,
                stroke: "hsl(var(--card))",
                strokeWidth: 2,
              }}
            />
          </LineChart>
        </ResponsiveContainer>
      </div>

      <div className="mt-3 flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
        <LegendDot
          tone="buy"
          label={firstOutcome.label}
          value={formatProbability(currentFirstPrice.toString())}
        />

        <div className="text-xs text-secondary">
          Updated: {new Date().toLocaleTimeString()}
        </div>

        <LegendDot
          tone="sell"
          label={secondOutcome.label}
          value={formatProbability(currentSecondPrice.toString())}
        />
      </div>
    </div>
  );
}

function LegendDot({
  tone,
  label,
  value,
}: {
  tone: "buy" | "sell";
  label: string;
  value: string;
}) {
  return (
    <span className="inline-flex items-center gap-1.5 rounded-md border border-border bg-surface px-2 py-1">
      <span
        className={`size-2 rounded-full ${
          tone === "buy" ? "bg-buy" : "bg-sell"
        }`}
      />
      <span className="text-sm font-medium text-text">{label}</span>
      <span
        className={`text-sm font-semibold tabular-nums ${
          tone === "buy" ? "text-buy" : "text-sell"
        }`}
      >
        {value}
      </span>
    </span>
  );
}
