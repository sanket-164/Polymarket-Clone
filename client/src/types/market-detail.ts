export interface Outcome {
    id: string
    market_id: string
    label: string
    start_price: string
    current_price: string
    total_shares: string
    created_at: string
    updated_at: string
}

export interface MarketDetail {
    id: string
    title: string
    description: string
    category: string
    start_at: string
    close_at: string
    status: string
    created_at: string
    updated_at: string
    deleted_at: string | null
    first_outcome: Outcome
    second_outcome: Outcome
}
