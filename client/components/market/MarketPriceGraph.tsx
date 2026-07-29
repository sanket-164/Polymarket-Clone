"use client";

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

// Note: Replace these hex codes with the actual hex values of your 'buy' and 'sell'
// Tailwind colors if they differ, to ensure perfect visual consistency with your theme.
const BUY_COLOR = "#10b981"; // emerald-500
const SELL_COLOR = "#ef4444"; // rose-500

export function MarketPriceGraph({
  firstOutcome,
  secondOutcome,
  priceHistory,
}: MarketPriceGraphProps) {
  const currentFirstPrice =
    priceHistory.length > 0
      ? priceHistory[priceHistory.length - 1].first
      : Number(firstOutcome.current_price);

  const currentSecondPrice =
    priceHistory.length > 0
      ? priceHistory[priceHistory.length - 1].second
      : Number(secondOutcome.current_price);

  const chartData =
    priceHistory.length > 0
      ? priceHistory
      : [
          {
            time: "Start",
            first: Number(firstOutcome.start_price),
            second: Number(secondOutcome.start_price),
          },
        ];

  return (
    <div className="rounded-2xl border border-border bg-surface p-6">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h2 className="text-lg font-semibold text-text">Price History</h2>
          <p className="text-sm text-secondary">
            Outcome probability over time
          </p>
        </div>
        <div className="flex items-center gap-3 text-sm">
          <LegendDot
            tone="buy"
            label={firstOutcome.label}
            value={formatProbability(currentFirstPrice.toString())}
          />
          <LegendDot
            tone="sell"
            label={secondOutcome.label}
            value={formatProbability(currentSecondPrice.toString())}
          />
        </div>
      </div>

      <div className="mt-6 h-72 w-full rounded-xl border border-border bg-card p-4">
        <ResponsiveContainer width="100%" height="100%">
          <LineChart
            data={chartData}
            margin={{ top: 5, right: 10, left: 0, bottom: 5 }}
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
            />
            <YAxis
              domain={[0, 1]}
              tickFormatter={(value) => `${Math.round(value * 100)}%`}
              fontSize={12}
              tickLine={false}
              axisLine={false}
              width={35}
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
                marginBottom: "4px",
              }}
              formatter={(value, _name) => {
                const numericValue =
                  typeof value === "number" ? value : Number(value);

                if (Number.isNaN(numericValue)) {
                  return ["-", ""];
                }

                return [
                  `${(numericValue * 100).toFixed(1)}%`,
                  _name === "first" ? firstOutcome.label : secondOutcome.label,
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
              activeDot={{
                r: 5,
                fill: BUY_COLOR,
                stroke: "hsl(var(--card))",
                strokeWidth: 2,
              }}
              animationDuration={300}
            />
            <Line
              type="monotone"
              dataKey="second"
              stroke={SELL_COLOR}
              strokeWidth={2}
              dot={false}
              activeDot={{
                r: 5,
                fill: SELL_COLOR,
                stroke: "hsl(var(--card))",
                strokeWidth: 2,
              }}
              animationDuration={300}
            />
          </LineChart>
        </ResponsiveContainer>
      </div>

      <div className="mt-4 flex items-center justify-center text-xs text-secondary">
        <span>Updated: {new Date().toLocaleTimeString()}</span>
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
    <span className="inline-flex items-center gap-2 rounded-lg border border-border bg-surface px-3 py-1.5">
      <span
        className={`size-2.5 rounded-full ${tone === "buy" ? "bg-buy" : "bg-sell"}`}
      />
      <span className="font-medium text-text">{label}</span>
      <span
        className={`font-semibold tabular-nums ${tone === "buy" ? "text-buy" : "text-sell"}`}
      >
        {value}
      </span>
    </span>
  );
}
