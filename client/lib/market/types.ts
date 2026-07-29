export type MarketStatus = "PENDING" | "ACTIVE" | "CLOSED" | "RESOLVED" | "CANCELLED";

export type Market = {
  id: string;
  title: string;
  description: string;
  category: string;
  start_at: string;
  close_at: string;
  status: MarketStatus;
  created_at: string;
  updated_at: string;
};

export type Outcome = {
  id: string;
  market_id: string;
  label: string;
  start_price: string;
  current_price: string;
  total_shares: string;
  created_at: string;
  updated_at: string;
};

export type MarketDetails = Market & {
  first_outcome: Outcome;
  second_outcome: Outcome;
};

export type OrderBookLevel = {
  price: string;
  qty: number;
};

export type OutcomeSnapshot = {
  buy: OrderBookLevel[];
  sell: OrderBookLevel[];
  outcome_id: string;
  outcome_label: string;
};

export type MarketSnapshot = {
  market_id: string;
  snapshot: OutcomeSnapshot[];
  updated_at: number;
};

export type MarketsQuery = {
  order_field?: "created_at" | "updated_at" | "start_at" | "close_at" | "title" | "category" | "status";
  order_by?: "ASC" | "DESC";
  category?: string;
  start_before?: string;
  start_after?: string;
  close_after?: string;
  close_before?: string;
  status?: MarketStatus | "";
};
