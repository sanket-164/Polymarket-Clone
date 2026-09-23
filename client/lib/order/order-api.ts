import { ORDER_SERVICE_URL } from "@/lib/api/config";
import { apiFetch } from "@/lib/api/http";
import type { OrderRequest, Order, OrderDetail, OrdersQuery } from "./types";

export async function placeOrder(request: OrderRequest): Promise<Order> {
    return apiFetch<Order>("/api/order", {
        baseUrl: ORDER_SERVICE_URL,
        method: "POST",
        body: JSON.stringify(request),
    });
}

export async function getOrders(query: OrdersQuery): Promise<Order[]> {
    const params = new URLSearchParams();

    if (query.market_id) {
        params.set("market_id", query.market_id);
    }
    if (query.outcome_id) {
        params.set("outcome_id", query.outcome_id);
    }
    if (query.side) {
        params.set("side", query.side);
    }
    if (query.order_type) {
        params.set("order_type", query.order_type);
    }
    if (query.status) {
        params.set("status", query.status);
    }
    if (query.order_field) {
        params.set("order_field", query.order_field);
    }
    if (query.order_by) {
        params.set("order_by", query.order_by);
    }
    if (query.limit !== undefined) {
        params.set("limit", String(query.limit));
    }
    if (query.skip !== undefined) {
        params.set("skip", String(query.skip));
    }
    if (query.before) {
        params.set("before", query.before);
    }
    if (query.after) {
        params.set("after", query.after);
    }

    const queryString = params.toString();
    const url = `/api/order${queryString ? `?${queryString}` : ""}`;

    return apiFetch<Order[]>(url, {
        baseUrl: ORDER_SERVICE_URL,
        method: "GET",
    });
}

export async function getOrder(orderId: string): Promise<OrderDetail> {
    return apiFetch<OrderDetail>(`/api/order/${encodeURIComponent(orderId)}`, {
        baseUrl: ORDER_SERVICE_URL,
        method: "GET",
    });
}

export async function cancelOrder(orderId: string): Promise<void> {
    await apiFetch(`/api/order/cancel/${encodeURIComponent(orderId)}`, {
        baseUrl: ORDER_SERVICE_URL,
        method: "PUT",
    });
}