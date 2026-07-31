export interface HoldingMarket {
    id: string;
    title: string;
    description: string;
    category: string;
    start_at: string;
    close_at: string;
    status: string;
    created_at: string;
    updated_at: string;
}

export interface HoldingOutcome {
    id: string;
    market_id: string;
    label: string;
    start_price: string;
    current_price: string;
    total_shares: string;
    created_at: string;
    updated_at: string;
}

export interface Holding {
    id: string;
    shares: string;
    locked_shares: string;
    created_at: string;
    updated_at: string;
    market: HoldingMarket;
    outcome: HoldingOutcome;
}

export type HoldingsQuery = {
    order_field?: "shares" | "created_at";
    order_by?: "ASC" | "DESC";
    limit?: number;
    skip?: number;
};
