import { useEffect, useState, useRef } from "react";
import type { OutcomeSnapshot, OrderBookLevel, MarketSnapshot } from "../lib/market/types";

interface OrderbookUpdate {
    market_id: string;
    outcome_id: string;
    side: "BUY" | "SELL";
    quantity: string;
    price: string;
    timestamp: number; // Ensure this matches the unit of MarketSnapshot.updated_at (e.g., milliseconds)
    trade: string | number;
}

export function useOrderbookWebSocket(
    marketId: string,
    fetchSnapshot: () => Promise<MarketSnapshot>,
    enabled: boolean
) {
    const [snapshots, setSnapshots] = useState<OutcomeSnapshot[]>([]);
    const [currentPrices, setCurrentPrices] = useState<Record<string, string>>({});
    const [isSynced, setIsSynced] = useState(false);

    const bufferRef = useRef<OrderbookUpdate[]>([]);
    const isSyncedRef = useRef(false);
    const wsRef = useRef<WebSocket | null>(null);

    // Keep track of the latest fetchSnapshot function without triggering effect re-runs
    const fetchSnapshotRef = useRef(fetchSnapshot);
    useEffect(() => {
        fetchSnapshotRef.current = fetchSnapshot;
    }, [fetchSnapshot]);

    useEffect(() => {
        if (!marketId || !enabled) return;

        let isCurrent = true;
        bufferRef.current = [];
        isSyncedRef.current = false;
        // eslint-disable-next-line react-hooks/set-state-in-effect
        setIsSynced(false);
        setSnapshots([]);
        setCurrentPrices({});

        const ws = new WebSocket(process.env.NEXT_PUBLIC_FEED_SERVICE_URL || "");
        wsRef.current = ws;

        ws.onopen = () => {
            ws.send(
                JSON.stringify({
                    type: "JoinMarket",
                    market_id: marketId,
                })
            );

            // Request snapshot while buffering incoming messages using the ref
            fetchSnapshotRef.current()
                .then((snapshot) => {
                    if (!isCurrent) return;

                    const snapshotTime = snapshot.updated_at;

                    // Filter buffer: keep only messages with timestamp > snapshotTime
                    const relevantUpdates = bufferRef.current.filter(
                        (msg) => msg.timestamp > snapshotTime
                    );

                    // Sort updates by timestamp to ensure correct order of application
                    relevantUpdates.sort((a, b) => a.timestamp - b.timestamp);

                    let currentSnapshots = snapshot.snapshot;

                    // Apply relevant updates to the snapshot
                    for (const update of relevantUpdates) {
                        currentSnapshots = applyUpdate(currentSnapshots, update);

                        updateCurrentPrice(update, setCurrentPrices);
                    }

                    setSnapshots(currentSnapshots);
                    isSyncedRef.current = true;
                    setIsSynced(true);
                    bufferRef.current = []; // Clear buffer after sync
                })
                .catch((err) => {
                    if (!isCurrent) return;
                    console.error("Failed to fetch snapshot:", err);
                    // Fallback: allow live updates even if snapshot fails to prevent infinite buffering
                    isSyncedRef.current = true;
                    setIsSynced(true);
                });
        };

        ws.onmessage = (event) => {
            try {
                const data: OrderbookUpdate = JSON.parse(event.data);

                if (data.market_id !== marketId) return;

                if (!isSyncedRef.current) {
                    bufferRef.current.push(data);
                } else {
                    if (isCurrent) {
                        setSnapshots((prev) => applyUpdate(prev, data));
                    }
                }

                if (isSyncedRef.current && isCurrent) {
                    updateCurrentPrice(data, setCurrentPrices);
                }
            } catch (err) {
                console.error("WebSocket message error:", err);
            }
        };

        ws.onerror = (err) => console.error("WebSocket error:", err);
        ws.onclose = () => {
            console.log("WebSocket closed");
            if (isCurrent) {
                isSyncedRef.current = false;
                setIsSynced(false);
            }
        };

        return () => {
            isCurrent = false;
            ws.close();
        };
        // Removed fetchSnapshot from dependencies to prevent reconnection on parent re-renders
    }, [marketId, enabled]);

    return { snapshots, currentPrices, isSynced };
}

function updateCurrentPrice(
    update: OrderbookUpdate,
    setCurrentPrices: React.Dispatch<React.SetStateAction<Record<string, string>>>
) {
    const tradePrice = Number(update.trade);

    if (!Number.isFinite(tradePrice) || tradePrice <= 0) return;

    setCurrentPrices((prev) => ({
        ...prev,
        [update.outcome_id]: String(tradePrice),
    }));
}

function applyUpdate(
    snapshots: OutcomeSnapshot[],
    data: OrderbookUpdate
): OutcomeSnapshot[] {
    return snapshots.map((snapshot) => {
        if (snapshot.outcome_id !== data.outcome_id) return snapshot;

        const qtyDelta = parseFloat(data.quantity);
        const priceNum = parseFloat(data.price);
        const isBuy = data.side === "BUY";

        const sourceArray = isBuy ? snapshot.buy : snapshot.sell;
        const targetArray: OrderBookLevel[] = sourceArray.map((l) => ({ ...l }));

        const existingIndex = targetArray.findIndex(
            (level) => parseFloat(level.price) === priceNum
        );

        if (existingIndex >= 0) {
            const newQty = Number(targetArray[existingIndex].qty) + qtyDelta;
            if (newQty <= 1e-8) {
                targetArray.splice(existingIndex, 1);
            } else {
                targetArray[existingIndex] = {
                    ...targetArray[existingIndex],
                    qty: newQty,
                };
            }
        } else if (qtyDelta > 0) {
            targetArray.push({ price: data.price, qty: qtyDelta });
        }

        return {
            ...snapshot,
            buy: isBuy ? targetArray : snapshot.buy,
            sell: !isBuy ? targetArray : snapshot.sell,
        };
    });
}