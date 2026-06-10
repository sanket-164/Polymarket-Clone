export interface OrderLevel {
    price: string
    qty: number
}

export interface OrderbookSnapshot {
    buy: OrderLevel[]
    outcome_id: string
    outcome_label: string
    sell: OrderLevel[]
}
