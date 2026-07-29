"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useState } from "react";
import { AuthCard } from "@/components/auth/AuthCard";
import { FormField } from "@/components/auth/FormField";
import { resetPassword, sendOtp } from "@/lib/auth/auth-api";
import { ApiError } from "@/lib/api/http";

type ResetMode = "otp" | "old-password";

export function ResetPasswordForm() {
  const router = useRouter();
  const [mode, setMode] = useState<ResetMode>("otp");
  const [otpEmail, setOtpEmail] = useState("");
  const [isOtpSent, setIsOtpSent] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [isSendingOtp, setIsSendingOtp] = useState(false);

  async function handleSendOtp(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setError(null);
    setMessage(null);
    setIsSendingOtp(true);

    const formData = new FormData(event.currentTarget);
    const email = String(formData.get("otpEmail"));

    try {
      const response = await sendOtp(email);
      setOtpEmail(email);
      setIsOtpSent(true);
      setMessage(response.message);
    } catch (caughtError) {
      setIsOtpSent(false);
      setError(getFormError(caughtError));
    } finally {
      setIsSendingOtp(false);
    }
  }

  async function handleResetPassword(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setError(null);
    setMessage(null);
    setIsSubmitting(true);

    const formData = new FormData(event.currentTarget);
    const email = mode === "otp" ? otpEmail : String(formData.get("email"));
    const newPassword = String(formData.get("newPassword"));
    const confirmPassword = String(formData.get("confirmPassword"));

    if (mode === "otp" && !isOtpSent) {
      setError("Send the OTP before resetting your password.");
      setIsSubmitting(false);
      return;
    }

    if (newPassword !== confirmPassword) {
      setError("Passwords do not match.");
      setIsSubmitting(false);
      return;
    }

    try {
      const response = await resetPassword(
        mode === "otp"
          ? {
              email,
              otp: Number(formData.get("otp")),
              new_password: newPassword,
              confirm_password: confirmPassword,
            }
          : {
              email,
              old_password: String(formData.get("oldPassword")),
              new_password: newPassword,
              confirm_password: confirmPassword,
            }
      );

      setMessage(response.message);
      router.push("/login");
    } catch (caughtError) {
      setError(getFormError(caughtError));
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <AuthCard eyebrow="Account recovery" title="Reset password">
      <div className="mt-6 grid grid-cols-2 gap-2 rounded-lg border border-border bg-card p-1">
        <button
          type="button"
          onClick={() => setMode("otp")}
          disabled={isSubmitting || isSendingOtp}
          className={getModeClassName(mode === "otp")}
        >
          OTP
        </button>
        <button
          type="button"
          onClick={() => {
            setMode("old-password");
            setError(null);
            setMessage(null);
          }}
          disabled={isSubmitting || isSendingOtp}
          className={getModeClassName(mode === "old-password")}
        >
          Old password
        </button>
      </div>

      {mode === "otp" && !isOtpSent ? (
        <form className="mt-5 flex gap-2" onSubmit={handleSendOtp}>
          <label htmlFor="otp-email" className="sr-only">
            Email for OTP
          </label>
          <input
            id="otp-email"
            name="otpEmail"
            type="email"
            required
            placeholder="Email"
            value={otpEmail}
            onChange={(event) => {
              setOtpEmail(event.target.value);
              setIsOtpSent(false);
              setMessage(null);
            }}
            disabled={isSendingOtp}
            className="h-11 min-w-0 flex-1 rounded-lg border border-border bg-card px-3 text-sm text-text outline-none transition placeholder:text-secondary focus:border-accent focus:ring-2 focus:ring-accent/25"
          />
          <button
            type="submit"
            disabled={isSendingOtp}
            className="h-11 rounded-lg border border-border bg-surface px-4 text-sm font-semibold text-text transition hover:bg-card disabled:opacity-40"
          >
            {isSendingOtp ? "Sending" : "Send OTP"}
          </button>
        </form>
      ) : null}

      <form className="mt-5 space-y-4" onSubmit={handleResetPassword}>
        {mode === "old-password" ? (
          <FormField
            id="reset-email"
            label="Email"
            name="email"
            type="email"
            autoComplete="email"
          />
        ) : null}

        {mode === "otp" && isOtpSent ? (
          <>
            <p className="rounded-lg border border-border bg-card px-3 py-2 text-sm text-secondary">
              OTP sent to{" "}
              <span className="font-semibold text-text">{otpEmail}</span>
            </p>
            <label htmlFor="reset-otp" className="block">
              <span className="text-sm font-medium text-text">OTP</span>
              <input
                id="reset-otp"
                name="otp"
                autoComplete="one-time-code"
                required
                disabled={isSubmitting}
                className="mt-2 h-11 w-full rounded-lg border border-border bg-card px-3 text-sm text-text outline-none transition placeholder:text-secondary focus:border-accent focus:ring-2 focus:ring-accent/25 disabled:opacity-40"
              />
            </label>
          </>
        ) : mode === "old-password" ? (
          <FormField
            id="reset-old-password"
            label="Old password"
            name="oldPassword"
            type="password"
            autoComplete="current-password"
          />
        ) : null}

        {mode === "otp" && isOtpSent ? (
          <>
            <PasswordField
              id="reset-new-password"
              label="New password"
              name="newPassword"
              disabled={isSubmitting}
            />
            <PasswordField
              id="reset-confirm-password"
              label="Confirm new password"
              name="confirmPassword"
              disabled={isSubmitting}
            />
          </>
        ) : mode === "old-password" ? (
          <>
            <PasswordField
              id="reset-new-password"
              label="New password"
              name="newPassword"
              disabled={isSubmitting}
            />
            <PasswordField
              id="reset-confirm-password"
              label="Confirm new password"
              name="confirmPassword"
              disabled={isSubmitting}
            />
          </>
        ) : null}

        {error ? <p className="text-sm text-sell">{error}</p> : null}
        {message ? <p className="text-sm text-buy">{message}</p> : null}

        <button
          type="submit"
          disabled={isSubmitting || (mode === "otp" && !isOtpSent)}
          className="h-11 w-full rounded-lg bg-accent px-4 text-sm font-semibold text-text transition hover:brightness-110 disabled:opacity-40"
        >
          {isSubmitting ? "Resetting..." : "Reset password"}
        </button>
      </form>

      <Link
        href="/login"
        className="mt-4 inline-block text-sm font-semibold text-accent"
      >
        Back to log in
      </Link>
    </AuthCard>
  );
}

function PasswordField({
  id,
  label,
  name,
  disabled,
}: {
  id: string;
  label: string;
  name: string;
  disabled: boolean;
}) {
  return (
    <label htmlFor={id} className="block">
      <span className="text-sm font-medium text-text">{label}</span>
      <input
        id={id}
        name={name}
        type="password"
        autoComplete="new-password"
        required
        minLength={8}
        disabled={disabled}
        className="mt-2 h-11 w-full rounded-lg border border-border bg-card px-3 text-sm text-text outline-none transition placeholder:text-secondary focus:border-accent focus:ring-2 focus:ring-accent/25 disabled:opacity-40"
      />
    </label>
  );
}

function getModeClassName(isActive: boolean) {
  return [
    "h-9 rounded-md text-sm font-semibold transition",
    isActive ? "bg-accent text-text" : "text-secondary hover:text-text",
  ].join(" ");
}

function getFormError(caughtError: unknown) {
  if (caughtError instanceof ApiError) {
    return caughtError.message;
  }

  return "Unable to reset password. Please try again.";
}
