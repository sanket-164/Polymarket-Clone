// lib/order/types.ts
export type OrderSide = "BUY" | "SELL";

export type OrderStatus = "PENDING" | "FILLED" | "CANCELLED" | "PARTIAL" | "EXPIRED";

export interface OrderRequest {
  market_id: string;
  outcome_id: string;
  shares: number;
  price: number;
  side: OrderSide;
  expires_at: string;
}

export interface Order {
  id: string;
  user_id: string;
  market_id: string;
  outcome_id: string;
  side: OrderSide;
  shares: string;
  remaining_shares: string;
  price: string;
  status: OrderStatus;
  created_at: string;
  updated_at: string;
  expires_at: string;
}

export interface OrderFormData {
  side: OrderSide;
  outcomeId: string;
  shares: number;
  price: number;
}

export type OrderSortField = "shares" | "price" | "created_at";

export type OrdersQuery = {
  market_id?: string;
  side?: OrderSide | "";
  status?: OrderStatus | "";
  order_field?: OrderSortField;
  order_by?: "ASC" | "DESC";
  limit?: number;
  skip?: number;
  before?: string;
  after?: string;
};