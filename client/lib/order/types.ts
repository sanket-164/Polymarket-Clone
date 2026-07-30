// lib/order/types.ts
export type OrderSide = "BUY" | "SELL";

export type OrderStatus = "PENDING" | "FILLED" | "CANCELLED" | "PARTIAL" | "EXPIRED";

export interface OrderRequest {
  market_id: string;
  outcome_id: string;
  shares: number;
  price: number;
  side: OrderSide;
}

export interface OrderResponse {
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
}

export interface OrderFormData {
  side: OrderSide;
  outcomeId: string;
  shares: number;
  price: number;
}