import type { User } from "@/lib/auth/types";

export type Profile = User;

export type UpdateProfilePayload = {
  name: string;
  email: string;
  picture: string | null;
  mobile_no: string | null;
};

export type UpdateProfilePicturePayload = {
  picture: string;
};

export type WalletBalance = {
  id: string;
  user_id: string;
  balance: string;
  locked_balance: string;
  created_at: string;
  updated_at: string;
};

export type WalletTransactionType = "BUY" | "SELL" | "DEPOSIT" | "WITHDRAW";

export type WalletTransaction = {
  id: string;
  wallet_id: string;
  type: WalletTransactionType | string;
  amount: string;
  created_at: string;
};

export type WalletTransactionsQuery = {
  order_by?: "ASC" | "DESC";
  order_field?: "created_at" | "amount" | "type";
  transaction_type?: WalletTransactionType | "";
  limit?: number;
  skip?: number;
};
