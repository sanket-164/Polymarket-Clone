import { apiFetch } from "@/lib/api/http";
import { BASE_API_URL } from "@/lib/api/config";
import type {
  Profile,
  UpdateProfilePayload,
  UpdateProfilePicturePayload,
  WalletBalance,
  WalletTransactionsQuery,
  WalletTransaction,
} from "@/lib/profile/types";

export async function getProfile() {
  return apiFetch<Profile>("/api/user/profile", {
    baseUrl: BASE_API_URL,
  });
}

export async function updateProfile(payload: UpdateProfilePayload) {
  return apiFetch<Profile>("/api/user/profile", {
    baseUrl: BASE_API_URL,
    method: "PUT",
    body: JSON.stringify(payload),
  });
}

export async function updateProfilePicture(payload: UpdateProfilePicturePayload) {
  return apiFetch<Profile>("/api/user/profile/picture", {
    baseUrl: BASE_API_URL,
    method: "PUT",
    body: JSON.stringify(payload),
  });
}

export async function getWalletBalance() {
  return apiFetch<WalletBalance>("/api/user/wallet/balance", {
    baseUrl: BASE_API_URL,
  });
}

export async function depositWallet(balance: number) {
  return apiFetch<WalletBalance>("/api/user/wallet/deposit", {
    baseUrl: BASE_API_URL,
    method: "POST",
    body: JSON.stringify({ balance }),
  });
}

export async function withdrawWallet(balance: number) {
  return apiFetch<WalletBalance>("/api/user/wallet/withdraw", {
    baseUrl: BASE_API_URL,
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
    `/api/user/wallet/transactions${queryString ? `?${queryString}` : ""}`,
    {
      baseUrl: BASE_API_URL,
    },
  );
}
