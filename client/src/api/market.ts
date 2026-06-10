import axios from 'axios'
import type { Market } from '../types/market'
import type { MarketDetail } from '../types/market-detail'
import type { OrderbookSnapshot } from '../types/orderbook'

const api = axios.create({
    baseURL: '',
})

const DEFAULT_MARKETS_QUERY = 'http://localhost:3000/api/market?start_after=2025-05-01T00:00:00Z'

export async function fetchMarkets(signal?: AbortSignal): Promise<Market[]> {
    const response = await api.get<Market[]>(DEFAULT_MARKETS_QUERY, { signal })
    return response.data
}

export async function fetchMarketById(
    marketId: string,
    signal?: AbortSignal,
): Promise<MarketDetail> {
    const response = await api.get<MarketDetail>(`http://localhost:3000/api/market/${marketId}`, { signal })
    return response.data
}

export async function fetchOrderbookSnapshot(
    marketId: string,
    signal?: AbortSignal,
): Promise<OrderbookSnapshot[]> {
    const response = await api.get<OrderbookSnapshot[]>(`http://localhost:5000/api/order/snapshot/${marketId}`, {
        signal,
    })
    return response.data
}
