import { USER_SERVICE_URL } from "@/lib/api/config";
import { apiFetch } from "@/lib/api/http";
import type { Holding, HoldingsQuery } from "./types";

export async function getHoldings(query: HoldingsQuery = {}): Promise<Holding[]> {
    const params = new URLSearchParams();

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

    const queryString = params.toString();
    const url = `/api/holding${queryString ? `?${queryString}` : ""}`;

    return apiFetch<Holding[]>(url, {
        baseUrl: USER_SERVICE_URL,
        method: "GET",
    });
}
