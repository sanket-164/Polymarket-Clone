// lib/order/types.ts
export type OrderSide = "BUY" | "SELL";

export type OrderType = "limit" | "market";
export type OrderQueryType = "LIMIT" | "MARKET";

export type OrderStatus = "PENDING" | "FILLED" | "CANCELLED" | "PARTIAL" | "EXPIRED";

export interface LimitOrderRequest {
  market_id: string;
  outcome_id: string;
  shares: number;
  price: number;
  side: OrderSide;
  order_type: "limit";
  expires_at: string;
}

export interface MarketBuyOrderRequest {
  market_id: string;
  outcome_id: string;
  side: "BUY";
  quote_amount: number;
  order_type: "market";
}

export interface MarketSellOrderRequest {
  market_id: string;
  outcome_id: string;
  side: "SELL";
  shares: number;
  price: "market";
  order_type: "market";
}

export type OrderRequest =
  | LimitOrderRequest
  | MarketBuyOrderRequest
  | MarketSellOrderRequest;

export interface Order {
  id: string;
  user_id: string;
  market_id: string;
  outcome_id: string;
  side: OrderSide;
  shares: string;
  remaining_shares: string;
  price: string;
  average_price: string;
  quote_amount?: string;
  remaining_quote?: string;
  order_type: OrderQueryType;
  status: OrderStatus;
  created_at: string;
  updated_at: string;
  expires_at: string;
}

export interface OrderTrade {
  id: string;
  market_id: string;
  buy_order_id: string;
  sell_order_id: string;
  shares: string;
  price: string;
  created_at: string;
}

export interface OrderDetail extends Order {
  quote_amount: string;
  remaining_quote: string;
  trade: OrderTrade[];
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
  outcome_id?: string;
  side?: OrderSide | "";
  status?: OrderStatus | "";
  order_type?: OrderQueryType | "";
  order_field?: OrderSortField;
  order_by?: "ASC" | "DESC";
  limit?: number;
  skip?: number;
  before?: string;
  after?: string;
};