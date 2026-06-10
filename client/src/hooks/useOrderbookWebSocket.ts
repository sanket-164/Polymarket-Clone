import { useEffect, useState, useRef } from "react";
import type { OrderbookSnapshot, OrderLevel } from "../types/orderbook";

interface OrderbookUpdate {
    market_id: string;
    outcome_id: string;
    side: "BUY" | "SELL";
    quantity: string;
    price: string;
}

export function useOrderbookWebSocket(
    marketId: string,
    initialSnapshots: OrderbookSnapshot[],
    enabled: boolean
) {
    const [snapshots, setSnapshots] = useState<OrderbookSnapshot[]>([]);
    const [currentPrices, setCurrentPrices] = useState<Record<string, string>>({});
    const initialSetRef = useRef(false);

    // Set initial snapshots when REST data becomes available
    useEffect(() => {
        if (enabled && initialSnapshots.length > 0 && !initialSetRef.current) {
            setSnapshots(initialSnapshots);
            initialSetRef.current = true;
        }
    }, [enabled, initialSnapshots]);

    // WebSocket connection
    useEffect(() => {
        if (!marketId || !enabled) return;

        const ws = new WebSocket(`ws://${window.location.host}/ws`);

        ws.onopen = () => {
            ws.send(
                JSON.stringify({
                    type: "JoinMarket",
                    market_id: marketId,
                })
            );
        };

        ws.onmessage = (event) => {
            try {
                const data: OrderbookUpdate = JSON.parse(event.data);

                if (data.market_id !== marketId) return;

                // Update current price on trade execution (SELL with negative quantity)
                if (data.side === "SELL" && parseFloat(data.quantity) < 0) {
                    setCurrentPrices((prev) => ({
                        ...prev,
                        [data.outcome_id]: data.price,
                    }));
                }

                // Update orderbook levels
                setSnapshots((prev) =>
                    prev.map((snapshot) => {
                        if (snapshot.outcome_id !== data.outcome_id) return snapshot;

                        const qtyDelta = parseFloat(data.quantity);
                        const priceNum = parseFloat(data.price);
                        const isBuy = data.side === "BUY";

                        const sourceArray = isBuy ? snapshot.buy : snapshot.sell;
                        const targetArray: OrderLevel[] = sourceArray.map((l) => ({ ...l }));

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
                    })
                );
            } catch (err) {
                console.error("WebSocket message error:", err);
            }
        };

        ws.onerror = (err) => console.error("WebSocket error:", err);
        ws.onclose = () => console.log("WebSocket closed");

        return () => {
            ws.close();
        };
    }, [marketId, enabled]);

    return { snapshots, currentPrices };
}