export type MarketStatus = 'ACTIVE' | 'CLOSED' | 'RESOLVED' | 'PENDING' | string

export interface Market {
    id: string
    title: string
    description: string
    category: string
    start_at: string
    close_at: string
    status: MarketStatus
    created_at: string
    updated_at: string
    deleted_at: string | null
}
