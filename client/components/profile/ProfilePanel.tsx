"use client";

import Image from "next/image";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { useEffect, useState } from "react";
import { useAuth } from "@/components/auth/AuthProvider";
import { logout } from "@/lib/auth/auth-api";
import { ApiError } from "@/lib/api/http";
import {
  getProfile,
  getWalletBalance,
  getWalletTransactions,
  updateProfile,
  updateProfilePicture,
} from "@/lib/profile/profile-api";
import { getOrders } from "@/lib/order/order-api";
import type {
  Profile,
  WalletBalance,
  WalletTransaction,
  WalletTransactionsQuery,
  WalletTransactionType,
} from "@/lib/profile/types";
import type {
  Order,
  OrdersQuery,
  OrderSide,
  OrderStatus,
} from "@/lib/order/types";

const MAX_PROFILE_IMAGE_SIZE = 1024 * 1024;
const DEFAULT_TRANSACTION_QUERY: Required<WalletTransactionsQuery> = {
  order_by: "DESC",
  order_field: "created_at",
  transaction_type: "",
  limit: 5,
  skip: 0,
};

const DEFAULT_ORDER_QUERY: Required<OrdersQuery> = {
  market_id: "",
  order_by: "DESC",
  order_field: "created_at",
  side: "",
  status: "",
  limit: 5,
  skip: 0,
  before: "",
  after: "",
};

export function ProfilePanel() {
  const router = useRouter();
  const { clearSession, isAuthenticated, isLoading } = useAuth();
  const [profile, setProfile] = useState<Profile | null>(null);
  const [balance, setBalance] = useState<WalletBalance | null>(null);
  const [transactions, setTransactions] = useState<WalletTransaction[]>([]);
  const [transactionQuery, setTransactionQuery] = useState(
    DEFAULT_TRANSACTION_QUERY
  );
  const [orders, setOrders] = useState<Order[]>([]);
  const [orderQuery, setOrderQuery] = useState(DEFAULT_ORDER_QUERY);
  const [showTransactions, setShowTransactions] = useState(false);
  const [showOrders, setShowOrders] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [isProfileLoading, setIsProfileLoading] = useState(true);
  const [isSavingProfile, setIsSavingProfile] = useState(false);
  const [isUploadingPicture, setIsUploadingPicture] = useState(false);
  const [isTransactionsLoading, setIsTransactionsLoading] = useState(true);
  const [isOrdersLoading, setIsOrdersLoading] = useState(true);
  const [isLoggingOut, setIsLoggingOut] = useState(false);

  useEffect(() => {
    if (!isLoading && isAuthenticated) {
      let isCurrent = true;

      Promise.all([getProfile(), getWalletBalance()])
        .then(([profileResponse, balanceResponse]) => {
          if (!isCurrent) {
            return;
          }

          setProfile(profileResponse);
          setBalance(balanceResponse);
        })
        .catch((caughtError: unknown) => {
          if (isCurrent) {
            setError(
              getPanelError(caughtError, "Unable to load your profile.")
            );
          }
        })
        .finally(() => {
          if (isCurrent) {
            setIsProfileLoading(false);
          }
        });

      return () => {
        isCurrent = false;
      };
    }
  }, [isAuthenticated, isLoading]);

  useEffect(() => {
    if (!isLoading && isAuthenticated && showTransactions) {
      let isCurrent = true;
      // eslint-disable-next-line react-hooks/set-state-in-effect
      setIsTransactionsLoading(true);

      getWalletTransactions(transactionQuery)
        .then((transactionResponse) => {
          if (isCurrent) {
            setTransactions(transactionResponse);
          }
        })
        .catch((caughtError: unknown) => {
          if (isCurrent) {
            setError(
              getPanelError(caughtError, "Unable to load wallet transactions.")
            );
          }
        })
        .finally(() => {
          if (isCurrent) {
            setIsTransactionsLoading(false);
          }
        });

      return () => {
        isCurrent = false;
      };
    }
  }, [isAuthenticated, isLoading, showTransactions, transactionQuery]);

  useEffect(() => {
    if (!isLoading && isAuthenticated && showOrders) {
      let isCurrent = true;
      // eslint-disable-next-line react-hooks/set-state-in-effect
      setIsOrdersLoading(true);

      getOrders(orderQuery)
        .then((orderResponse) => {
          if (isCurrent) {
            setOrders(orderResponse);
          }
        })
        .catch((caughtError: unknown) => {
          if (isCurrent) {
            setError(getPanelError(caughtError, "Unable to load orders."));
          }
        })
        .finally(() => {
          if (isCurrent) {
            setIsOrdersLoading(false);
          }
        });

      return () => {
        isCurrent = false;
      };
    }
  }, [isAuthenticated, isLoading, showOrders, orderQuery]);

  async function handleLogout() {
    setIsLoggingOut(true);

    try {
      await logout();
    } finally {
      clearSession();
      router.push("/login");
    }
  }

  async function handleProfileSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();

    if (!profile) {
      return;
    }

    setError(null);
    setSuccess(null);
    setIsSavingProfile(true);

    const formData = new FormData(event.currentTarget);

    try {
      const updatedProfile = await updateProfile({
        name: String(formData.get("name") ?? "").trim(),
        email: String(formData.get("email") ?? "").trim(),
        picture: profile.picture,
        mobile_no: normalizeOptionalValue(formData.get("mobile_no")),
      });

      setProfile(updatedProfile);
      setSuccess("Profile updated.");
    } catch (caughtError) {
      setError(getPanelError(caughtError, "Unable to update your profile."));
    } finally {
      setIsSavingProfile(false);
    }
  }

  async function handlePictureChange(
    event: React.ChangeEvent<HTMLInputElement>
  ) {
    const file = event.target.files?.[0];

    if (!file) {
      return;
    }

    setError(null);
    setSuccess(null);

    if (file.size > MAX_PROFILE_IMAGE_SIZE) {
      setError("Profile picture must be 1 MB or smaller.");
      event.target.value = "";
      return;
    }

    if (!file.type.startsWith("image/")) {
      setError("Please upload an image file.");
      event.target.value = "";
      return;
    }

    setIsUploadingPicture(true);

    try {
      const picture = await fileToBase64(file);
      const updatedProfile = await updateProfilePicture({ picture });

      setProfile(updatedProfile);
      setSuccess("Profile picture updated.");
    } catch (caughtError) {
      setError(
        getPanelError(caughtError, "Unable to update your profile picture.")
      );
    } finally {
      setIsUploadingPicture(false);
      event.target.value = "";
    }
  }

  function handleTransactionFilterChange(
    key: keyof Required<WalletTransactionsQuery>,
    value: string
  ) {
    setIsTransactionsLoading(true);
    setTransactionQuery((currentQuery) => ({
      ...currentQuery,
      [key]: key === "limit" || key === "skip" ? Number(value) : value,
      ...(key === "transaction_type" ? { skip: 0 } : null),
    }));
  }

  function handlePreviousTransactions() {
    setIsTransactionsLoading(true);
    setTransactionQuery((currentQuery) => ({
      ...currentQuery,
      skip: Math.max(0, currentQuery.skip - currentQuery.limit),
    }));
  }

  function handleNextTransactions() {
    setIsTransactionsLoading(true);
    setTransactionQuery((currentQuery) => ({
      ...currentQuery,
      skip: currentQuery.skip + currentQuery.limit,
    }));
  }

  function handleOrderFilterChange(
    key: keyof Required<OrdersQuery>,
    value: string
  ) {
    setIsOrdersLoading(true);
    setOrderQuery((currentQuery) => ({
      ...currentQuery,
      [key]: key === "limit" || key === "skip" ? Number(value) : value,
      ...(key === "side" || key === "status" ? { skip: 0 } : null),
    }));
  }

  function handlePreviousOrders() {
    setIsOrdersLoading(true);
    setOrderQuery((currentQuery) => ({
      ...currentQuery,
      skip: Math.max(0, currentQuery.skip - currentQuery.limit),
    }));
  }

  function handleNextOrders() {
    setIsOrdersLoading(true);
    setOrderQuery((currentQuery) => ({
      ...currentQuery,
      skip: currentQuery.skip + currentQuery.limit,
    }));
  }

  if (isLoading) {
    return (
      <ProfileShell
        title="Loading profile"
        description="Checking your session..."
      />
    );
  }

  if (!isAuthenticated) {
    router.push("/");
  }

  return (
    <section className="mx-auto w-full max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
      <div className="rounded-2xl border border-border bg-surface p-4 sm:p-6">
        <div className="flex flex-col gap-6 sm:flex-row sm:items-center sm:justify-between">
          <div className="flex items-center gap-4">
            <Image
              src={profile?.picture || "/default-profile.svg"}
              alt="Profile"
              width={56}
              height={56}
              unoptimized={Boolean(profile?.picture)}
              className="size-14 rounded-full border border-border bg-card object-cover"
            />
            <div>
              <p className="text-sm text-secondary">Profile</p>
              <h1 className="mt-1 text-2xl font-bold text-text sm:text-3xl">
                {profile?.name ?? "Your account"}
              </h1>
            </div>
          </div>

          <div className="flex flex-col gap-2 sm:flex-row">
            <Link
              href="/reset-password"
              className="inline-flex h-11 items-center justify-center rounded-lg border border-border bg-card px-4 text-sm font-semibold text-text transition hover:border-accent"
            >
              Reset password
            </Link>
            <button
              type="button"
              onClick={handleLogout}
              disabled={isLoggingOut}
              className="h-11 rounded-lg border border-border bg-card px-4 text-sm font-semibold text-text transition hover:border-sell disabled:opacity-40"
            >
              {isLoggingOut ? "Logging out..." : "Log out"}
            </button>
          </div>
        </div>

        {error ? (
          <p className="mt-6 rounded-lg border border-accent/40 bg-card px-4 py-3 text-sm text-secondary">
            {error}
          </p>
        ) : null}

        {success ? (
          <p className="mt-6 rounded-lg border border-buy/40 bg-card px-4 py-3 text-sm text-buy">
            {success}
          </p>
        ) : null}

        {isProfileLoading ? (
          <div className="mt-6 rounded-xl border border-border bg-card p-4 text-sm text-secondary">
            Loading account details...
          </div>
        ) : (
          <div className="mt-6 grid gap-4 lg:grid-cols-[minmax(0,1fr)_360px]">
            <form
              className="rounded-xl border border-border bg-card p-4 sm:p-5"
              onSubmit={handleProfileSubmit}
            >
              <div className="flex items-center justify-between gap-4">
                <div>
                  <h2 className="text-lg font-semibold text-text">
                    Account details
                  </h2>
                  <p className="mt-1 text-sm text-secondary">
                    Manage your public profile data.
                  </p>
                </div>
                <span className="rounded-full border border-border bg-surface px-3 py-1 text-xs font-medium text-buy">
                  Active
                </span>
              </div>

              <div className="mt-5 grid gap-4 sm:grid-cols-2">
                <ProfileField
                  id="profile-name"
                  label="Name"
                  name="name"
                  defaultValue={profile?.name ?? ""}
                />
                <ProfileField
                  id="profile-email"
                  label="Email"
                  name="email"
                  type="email"
                  defaultValue={profile?.email ?? ""}
                />
                <ProfileField
                  id="profile-mobile"
                  label="Mobile number"
                  name="mobile_no"
                  type="tel"
                  required={false}
                  defaultValue={profile?.mobile_no ?? ""}
                />
                <div>
                  <p className="text-sm font-medium text-text">Picture</p>
                  <label
                    htmlFor="profile-picture"
                    className="mt-2 flex h-11 cursor-pointer items-center justify-center rounded-lg border border-border bg-surface px-3 text-sm font-semibold text-text transition hover:border-accent"
                  >
                    {isUploadingPicture ? "Uploading..." : "Upload image"}
                  </label>
                  <input
                    id="profile-picture"
                    name="picture"
                    type="file"
                    accept="image/*"
                    disabled={isUploadingPicture}
                    onChange={handlePictureChange}
                    className="sr-only"
                  />
                  <p className="mt-2 text-xs text-secondary">
                    Maximum file size: 1 MB.
                  </p>
                </div>
              </div>

              <div className="mt-5 grid gap-3 border-t border-border pt-4 sm:grid-cols-2">
                <MetaItem
                  label="Created"
                  value={formatDateTime(profile?.created_at)}
                />

                <button
                  type="submit"
                  disabled={isSavingProfile}
                  className="h-11 w-full rounded-lg bg-accent px-4 text-sm font-semibold text-text transition hover:brightness-110 disabled:opacity-40 sm:w-auto"
                >
                  {isSavingProfile ? "Saving..." : "Save changes"}
                </button>
              </div>
            </form>

            <div className="rounded-xl border border-border bg-card p-4 sm:p-5">
              <h2 className="text-lg font-semibold text-text">
                Wallet balance
              </h2>
              <dl className="mt-5 grid gap-4">
                <BalanceItem
                  label="Available balance"
                  value={formatCurrency(balance?.balance)}
                  tone="buy"
                />
                <BalanceItem
                  label="Locked balance"
                  value={formatCurrency(balance?.locked_balance)}
                  tone="sell"
                />
              </dl>
            </div>
          </div>
        )}

        <div className="mt-6 flex flex-wrap justify-center gap-4">
          <button
            type="button"
            onClick={() => setShowTransactions((current) => !current)}
            className="h-11 rounded-lg border border-border bg-card px-6 text-sm font-semibold text-text transition hover:border-accent"
          >
            {showTransactions
              ? "Hide wallet transactions"
              : "View wallet transactions"}
          </button>
          <button
            type="button"
            onClick={() => setShowOrders((current) => !current)}
            className="h-11 rounded-lg border border-border bg-card px-6 text-sm font-semibold text-text transition hover:border-accent"
          >
            {showOrders ? "Hide recent orders" : "View recent orders"}
          </button>
        </div>

        {showTransactions && (
          <div className="mt-4 rounded-xl border border-border bg-card p-4 sm:p-5">
            <div className="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
              <div>
                <h2 className="text-lg font-semibold text-text">
                  Wallet transactions
                </h2>
                <p className="mt-1 text-sm text-secondary">
                  Review recent balance movements.
                </p>
              </div>
              <div className="grid gap-2 sm:grid-cols-3 lg:w-[520px]">
                <SelectField
                  id="transaction-type"
                  label="Type"
                  value={transactionQuery.transaction_type}
                  onChange={(value) =>
                    handleTransactionFilterChange("transaction_type", value)
                  }
                  options={[
                    { label: "All", value: "" },
                    { label: "Buy", value: "BUY" },
                    { label: "Sell", value: "SELL" },
                    { label: "Deposit", value: "DEPOSIT" },
                    { label: "Withdraw", value: "WITHDRAW" },
                  ]}
                />
                <SelectField
                  id="transaction-order"
                  label="Order"
                  value={transactionQuery.order_by}
                  onChange={(value) =>
                    handleTransactionFilterChange("order_by", value)
                  }
                  options={[
                    { label: "Newest", value: "DESC" },
                    { label: "Oldest", value: "ASC" },
                  ]}
                />
                <SelectField
                  id="transaction-limit"
                  label="Limit"
                  value={String(transactionQuery.limit)}
                  onChange={(value) =>
                    handleTransactionFilterChange("limit", value)
                  }
                  options={[
                    { label: "5", value: "5" },
                    { label: "10", value: "10" },
                    { label: "15", value: "15" },
                  ]}
                />
              </div>
            </div>

            <div className="mt-5 overflow-x-auto">
              <table className="w-full min-w-[640px] border-collapse text-left text-sm">
                <thead className="bg-surface text-xs uppercase text-secondary">
                  <tr>
                    <th className="border-b border-border px-3 py-3 font-medium">
                      Type
                    </th>
                    <th className="border-b border-border px-3 py-3 font-medium">
                      Amount
                    </th>
                    <th className="border-b border-border px-3 py-3 font-medium">
                      Transaction ID
                    </th>
                    <th className="border-b border-border px-3 py-3 font-medium">
                      Created
                    </th>
                  </tr>
                </thead>
                <tbody>
                  {isTransactionsLoading ? (
                    <tr>
                      <td
                        colSpan={4}
                        className="px-3 py-6 text-center text-secondary"
                      >
                        Loading transactions...
                      </td>
                    </tr>
                  ) : transactions.length > 0 ? (
                    transactions.map((transaction) => (
                      <tr
                        key={transaction.id}
                        className="transition hover:bg-surface"
                      >
                        <td className="border-b border-border px-3 py-3">
                          <TransactionTypeBadge type={transaction.type} />
                        </td>
                        <td className="border-b border-border px-3 py-3 font-mono text-text">
                          {formatCurrency(transaction.amount)}
                        </td>
                        <td className="border-b border-border px-3 py-3 font-mono text-xs text-secondary">
                          {transaction.id}
                        </td>
                        <td className="border-b border-border px-3 py-3 text-secondary">
                          {formatDateTime(transaction.created_at)}
                        </td>
                      </tr>
                    ))
                  ) : (
                    <tr>
                      <td
                        colSpan={4}
                        className="px-3 py-6 text-center text-secondary"
                      >
                        No transactions found.
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
            </div>

            <div className="mt-4 flex flex-col gap-2 sm:flex-row sm:justify-end">
              <button
                type="button"
                disabled={transactionQuery.skip === 0 || isTransactionsLoading}
                onClick={handlePreviousTransactions}
                className="h-10 rounded-lg border border-border bg-surface px-4 text-sm font-semibold text-text transition hover:border-accent disabled:opacity-40"
              >
                Previous
              </button>
              <button
                type="button"
                disabled={
                  transactions.length < transactionQuery.limit ||
                  isTransactionsLoading
                }
                onClick={handleNextTransactions}
                className="h-10 rounded-lg border border-border bg-surface px-4 text-sm font-semibold text-text transition hover:border-accent disabled:opacity-40"
              >
                Next
              </button>
            </div>
          </div>
        )}

        {showOrders && (
          <div className="mt-4 rounded-xl border border-border bg-card p-4 sm:p-5">
            <div className="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
              <div>
                <h2 className="text-lg font-semibold text-text">
                  Recent orders
                </h2>
                <p className="mt-1 text-sm text-secondary">
                  Track your buy and sell orders.
                </p>
              </div>
              <div className="grid gap-2 sm:grid-cols-2 lg:grid-cols-3 lg:w-[780px]">
                <SelectField
                  id="order-side"
                  label="Side"
                  value={orderQuery.side}
                  onChange={(value) => handleOrderFilterChange("side", value)}
                  options={[
                    { label: "All", value: "" },
                    { label: "Buy", value: "BUY" },
                    { label: "Sell", value: "SELL" },
                  ]}
                />
                <SelectField
                  id="order-status"
                  label="Status"
                  value={orderQuery.status}
                  onChange={(value) => handleOrderFilterChange("status", value)}
                  options={[
                    { label: "All", value: "" },
                    { label: "Pending", value: "PENDING" },
                    { label: "Filled", value: "FILLED" },
                    { label: "Partially filled", value: "PARTIAL" },
                    { label: "Cancelled", value: "CANCELLED" },
                    { label: "Expired", value: "EXPIRED" },
                  ]}
                />
                <SelectField
                  id="order-field"
                  label="Sort by"
                  value={orderQuery.order_field}
                  onChange={(value) =>
                    handleOrderFilterChange("order_field", value)
                  }
                  options={[
                    { label: "Created", value: "created_at" },
                    { label: "Price", value: "price" },
                    { label: "Shares", value: "shares" },
                  ]}
                />
                <SelectField
                  id="order-direction"
                  label="Direction"
                  value={orderQuery.order_by}
                  onChange={(value) =>
                    handleOrderFilterChange("order_by", value)
                  }
                  options={[
                    { label: "Descending", value: "DESC" },
                    { label: "Ascending", value: "ASC" },
                  ]}
                />
                <SelectField
                  id="order-limit"
                  label="Limit"
                  value={String(orderQuery.limit)}
                  onChange={(value) => handleOrderFilterChange("limit", value)}
                  options={[
                    { label: "5", value: "5" },
                    { label: "10", value: "10" },
                    { label: "15", value: "15" },
                  ]}
                />
              </div>
            </div>

            <div className="mt-5 overflow-x-auto">
              <table className="w-full min-w-[800px] border-collapse text-left text-sm">
                <thead className="bg-surface text-xs uppercase text-secondary">
                  <tr>
                    <th className="border-b border-border px-3 py-3 font-medium">
                      Side
                    </th>
                    <th className="border-b border-border px-3 py-3 font-medium">
                      Shares
                    </th>
                    <th className="border-b border-border px-3 py-3 font-medium">
                      Price
                    </th>
                    <th className="border-b border-border px-3 py-3 font-medium">
                      Status
                    </th>
                    <th className="border-b border-border px-3 py-3 font-medium">
                      Market
                    </th>
                    <th className="border-b border-border px-3 py-3 font-medium">
                      Order ID
                    </th>
                    <th className="border-b border-border px-3 py-3 font-medium">
                      Created
                    </th>
                  </tr>
                </thead>
                <tbody>
                  {isOrdersLoading ? (
                    <tr>
                      <td
                        colSpan={6}
                        className="px-3 py-6 text-center text-secondary"
                      >
                        Loading orders...
                      </td>
                    </tr>
                  ) : orders.length > 0 ? (
                    orders.map((order) => (
                      <tr
                        key={order.id}
                        className="transition hover:bg-surface"
                      >
                        <td className="border-b border-border px-3 py-3">
                          <OrderSideBadge side={order.side} />
                        </td>
                        <td className="border-b border-border px-3 py-3 font-mono text-text">
                          <div>{formatShares(order.shares)}</div>
                          <div className="text-xs text-secondary">
                            {formatShares(order.remaining_shares)} remaining
                          </div>
                        </td>
                        <td className="border-b border-border px-3 py-3 font-mono text-text">
                          {formatCurrency(order.price)}
                        </td>
                        <td className="border-b border-border px-3 py-3">
                          <OrderStatusBadge status={order.status} />
                        </td>
                        <td className="border-b border-border px-3 py-3">
                          <Link
                            href={`/markets/${order.market_id}`}
                            className="inline-flex h-9 items-center justify-center rounded-lg border border-border bg-surface px-3 text-sm font-semibold text-text transition hover:border-accent"
                          >
                            Open market
                          </Link>
                        </td>
                        <td className="border-b border-border px-3 py-3 font-mono text-xs text-secondary">
                          {order.id}
                        </td>
                        <td className="border-b border-border px-3 py-3 text-secondary">
                          {formatDateTime(order.created_at)}
                        </td>
                      </tr>
                    ))
                  ) : (
                    <tr>
                      <td
                        colSpan={6}
                        className="px-3 py-6 text-center text-secondary"
                      >
                        No orders found.
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
            </div>

            <div className="mt-4 flex flex-col gap-2 sm:flex-row sm:justify-end">
              <button
                type="button"
                disabled={orderQuery.skip === 0 || isOrdersLoading}
                onClick={handlePreviousOrders}
                className="h-10 rounded-lg border border-border bg-surface px-4 text-sm font-semibold text-text transition hover:border-accent disabled:opacity-40"
              >
                Previous
              </button>
              <button
                type="button"
                disabled={orders.length < orderQuery.limit || isOrdersLoading}
                onClick={handleNextOrders}
                className="h-10 rounded-lg border border-border bg-surface px-4 text-sm font-semibold text-text transition hover:border-accent disabled:opacity-40"
              >
                Next
              </button>
            </div>
          </div>
        )}
      </div>
    </section>
  );
}

type ProfileFieldProps = {
  id: string;
  label: string;
  name: string;
  type?: string;
  defaultValue: string;
  required?: boolean;
};

function ProfileField({
  id,
  label,
  name,
  type = "text",
  defaultValue,
  required = true,
}: ProfileFieldProps) {
  return (
    <label htmlFor={id} className="block">
      <span className="text-sm font-medium text-text">{label}</span>
      <input
        id={id}
        name={name}
        type={type}
        required={required}
        defaultValue={defaultValue}
        className="mt-2 h-11 w-full rounded-lg border border-border bg-surface px-3 text-sm text-text outline-none transition placeholder:text-secondary focus:border-accent focus:ring-2 focus:ring-accent/25"
      />
    </label>
  );
}

type SelectFieldProps = {
  id: string;
  label: string;
  value: string;
  options: Array<{ label: string; value: string }>;
  onChange: (value: string) => void;
};

function SelectField({
  id,
  label,
  value,
  options,
  onChange,
}: SelectFieldProps) {
  return (
    <label htmlFor={id} className="block">
      <span className="text-xs font-medium uppercase text-secondary">
        {label}
      </span>
      <select
        id={id}
        value={value}
        onChange={(event) => onChange(event.target.value)}
        className="mt-2 h-10 w-full rounded-lg border border-border bg-surface px-3 text-sm text-text outline-none transition focus:border-accent focus:ring-2 focus:ring-accent/25"
      >
        {options.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
    </label>
  );
}

function MetaItem({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <dt className="text-xs uppercase text-secondary">{label}</dt>
      <dd className="mt-1 break-all font-mono text-sm text-text">{value}</dd>
    </div>
  );
}

function BalanceItem({
  label,
  value,
  tone,
}: {
  label: string;
  value: string;
  tone?: "buy" | "sell";
}) {
  const toneClassName =
    tone === "buy" ? "text-buy" : tone === "sell" ? "text-sell" : "text-text";

  return (
    <div className="rounded-lg border border-border bg-surface p-4">
      <dt className="text-xs uppercase text-secondary">{label}</dt>
      <dd
        className={`mt-2 font-mono text-2xl font-bold tabular-nums ${toneClassName}`}
      >
        {value}
      </dd>
    </div>
  );
}

function TransactionTypeBadge({ type }: { type: string }) {
  const normalizedType = type.toUpperCase() as WalletTransactionType;
  const className =
    normalizedType === "BUY" || normalizedType === "DEPOSIT"
      ? "border-buy/30 bg-buy/15 text-buy"
      : normalizedType === "SELL" || normalizedType === "WITHDRAW"
        ? "border-sell/30 bg-sell/15 text-sell"
        : "border-border bg-surface text-secondary";

  return (
    <span
      className={`rounded-full border px-2.5 py-1 text-xs font-semibold ${className}`}
    >
      {type}
    </span>
  );
}

function OrderSideBadge({ side }: { side: string }) {
  const normalizedSide = side.toUpperCase() as OrderSide;
  const className =
    normalizedSide === "BUY"
      ? "border-buy/30 bg-buy/15 text-buy"
      : normalizedSide === "SELL"
        ? "border-sell/30 bg-sell/15 text-sell"
        : "border-border bg-surface text-secondary";

  return (
    <span
      className={`rounded-full border px-2.5 py-1 text-xs font-semibold ${className}`}
    >
      {side}
    </span>
  );
}

function OrderStatusBadge({ status }: { status: string }) {
  const normalizedStatus = status.toUpperCase() as OrderStatus;
  const className =
    normalizedStatus === "FILLED"
      ? "border-buy/30 bg-buy/15 text-buy"
      : normalizedStatus === "CANCELLED"
        ? "border-sell/30 bg-sell/15 text-sell"
        : normalizedStatus === "PARTIAL"
          ? "border-accent/30 bg-accent/15 text-accent"
          : "border-border bg-surface text-secondary";

  return (
    <span
      className={`rounded-full border px-2.5 py-1 text-xs font-semibold ${className}`}
    >
      {status.replace("_", " ")}
    </span>
  );
}

function ProfileShell({
  title,
  description,
}: {
  title: string;
  description: string;
}) {
  return (
    <section className="mx-auto w-full max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
      <div className="rounded-2xl border border-border bg-surface p-6">
        <p className="text-sm text-secondary">Profile</p>
        <h1 className="mt-2 text-3xl font-bold text-text">{title}</h1>
        <p className="mt-2 text-sm text-secondary">{description}</p>
      </div>
    </section>
  );
}

function normalizeOptionalValue(value: FormDataEntryValue | null) {
  const normalizedValue = String(value ?? "").trim();

  return normalizedValue ? normalizedValue : null;
}

function fileToBase64(file: File) {
  return new Promise<string>((resolve, reject) => {
    const reader = new FileReader();

    reader.onload = () => resolve(String(reader.result));
    reader.onerror = () =>
      reject(new Error("Unable to read the selected image."));
    reader.readAsDataURL(file);
  });
}

function formatCurrency(value: string | number | null | undefined) {
  if (value === null || value === undefined || Number.isNaN(Number(value))) {
    return "--";
  }

  return new Intl.NumberFormat("en-US", {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(Number(value));
}

function formatShares(value: string | number | null | undefined) {
  if (value === null || value === undefined || Number.isNaN(Number(value))) {
    return "--";
  }

  return new Intl.NumberFormat("en-US", {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(Number(value));
}

function formatDateTime(value: string | null | undefined) {
  if (!value) {
    return "Unavailable";
  }

  return new Intl.DateTimeFormat("en-US", {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(value));
}

function getPanelError(caughtError: unknown, fallback: string) {
  if (caughtError instanceof ApiError) {
    return caughtError.message;
  }

  if (caughtError instanceof Error) {
    return caughtError.message;
  }

  return fallback;
}
