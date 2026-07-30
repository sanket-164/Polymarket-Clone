import { ORDER_SERVICE_URL } from "@/lib/api/config";
import { apiFetch } from "@/lib/api/http";
import type { OrderRequest, OrderResponse } from "./types";

export async function placeOrder(request: OrderRequest): Promise<OrderResponse> {
    return apiFetch<OrderResponse>("/api/order", {
        baseUrl: ORDER_SERVICE_URL,
        method: "POST",
        body: JSON.stringify(request),
    });
}