import { apiFetch } from "@/lib/api/http";
import { USER_SERVICE_URL } from "@/lib/api/config";
import type {
  Profile,
  UpdateProfilePayload,
  UpdateProfilePicturePayload,
  WalletBalance,
  WalletTransactionsQuery,
  WalletTransaction,
} from "@/lib/profile/types";

export async function getProfile() {
  return apiFetch<Profile>("/api/profile", {
    baseUrl: USER_SERVICE_URL,
  });
}

export async function updateProfile(payload: UpdateProfilePayload) {
  return apiFetch<Profile>("/api/profile", {
    baseUrl: USER_SERVICE_URL,
    method: "PUT",
    body: JSON.stringify(payload),
  });
}

export async function updateProfilePicture(payload: UpdateProfilePicturePayload) {
  return apiFetch<Profile>("/api/profile/picture", {
    baseUrl: USER_SERVICE_URL,
    method: "PUT",
    body: JSON.stringify(payload),
  });
}

export async function getWalletBalance() {
  return apiFetch<WalletBalance>("/api/wallet/balance", {
    baseUrl: USER_SERVICE_URL,
  });
}

export async function depositWallet(balance: number) {
  return apiFetch<WalletBalance>("/api/wallet/deposit", {
    baseUrl: USER_SERVICE_URL,
    method: "POST",
    body: JSON.stringify({ balance }),
  });
}

export async function withdrawWallet(balance: number) {
  return apiFetch<WalletBalance>("/api/wallet/withdraw", {
    baseUrl: USER_SERVICE_URL,
    method: "POST",
    body: JSON.stringify({ balance }),
  });
}

export async function getWalletTransactions(query: WalletTransactionsQuery = {}) {
  const searchParams = new URLSearchParams();

  Object.entries(query).forEach(([key, value]) => {
    if (value !== undefined && value !== "") {
      searchParams.set(key, String(value));
    }
  });

  const queryString = searchParams.toString();

  return apiFetch<WalletTransaction[]>(
    `/api/wallet/transactions${queryString ? `?${queryString}` : ""}`,
    {
      baseUrl: USER_SERVICE_URL,
    },
  );
}
