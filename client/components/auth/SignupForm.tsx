"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useState } from "react";
import { AuthCard } from "@/components/auth/AuthCard";
import { FormField } from "@/components/auth/FormField";
import { useToast } from "@/components/toast/ToastProvider";
import { sendOtp, signUp, verifySignup } from "@/lib/auth/auth-api";
import { ApiError } from "@/lib/api/http";
import type { SignUpPayload } from "@/lib/auth/types";

export function SignupForm() {
  const router = useRouter();
  const { error: showError, success: showSuccess } = useToast();
  const [signupDetails, setSignupDetails] = useState<SignUpPayload | null>(null);
  const [isOtpSent, setIsOtpSent] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);

  async function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setIsSubmitting(true);

    const formData = new FormData(event.currentTarget);
    const password = signupDetails?.password ?? String(formData.get("password"));
    const confirmPassword = signupDetails?.confirmPassword ?? String(formData.get("confirmPassword"));

    if (!isOtpSent && password !== confirmPassword) {
      const message = "Passwords do not match.";
      showError(message);
      setIsSubmitting(false);
      return;
    }

    try {
      if (!isOtpSent) {
        const details = {
          name: String(formData.get("name")),
          email: String(formData.get("email")),
          password,
          confirmPassword,
        };
        const response = await sendOtp(details.email, "signup");
        setSignupDetails(details);
        setIsOtpSent(true);
        showSuccess(response.message || "OTP sent to your email.");
        return;
      }

      const email = signupDetails?.email;
      if (!email) {
        showError("Please request a new signup verification code.");
        setIsOtpSent(false);
        return;
      }

      await verifySignup({
        email,
        otp: Number(formData.get("otp")),
      });
      await signUp(signupDetails);

      showSuccess("Your account was created successfully.");
      router.push("/login");
    } catch (caughtError) {
      const message = getFormError(caughtError);
      showError(message);
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <AuthCard eyebrow="Create an account" title="Sign up">
      <form className="mt-6 space-y-4" onSubmit={handleSubmit}>
        <FormField
          id="signup-name"
          label="Name"
          name="name"
          autoComplete="name"
          disabled={isOtpSent || isSubmitting}
        />
        <FormField
          id="signup-email"
          label="Email"
          name="email"
          type="email"
          autoComplete="email"
          disabled={isOtpSent || isSubmitting}
        />
        <FormField
          id="signup-password"
          label="Password"
          name="password"
          type="password"
          autoComplete="new-password"
          minLength={8}
          disabled={isOtpSent || isSubmitting}
        />
        <FormField
          id="signup-confirm-password"
          label="Confirm password"
          name="confirmPassword"
          type="password"
          autoComplete="new-password"
          minLength={8}
          disabled={isOtpSent || isSubmitting}
        />

        {isOtpSent ? (
          <>
            <p className="rounded-lg border border-border bg-card px-3 py-2 text-sm text-secondary">
              OTP sent to{" "}
              <span className="font-semibold text-text">{signupDetails?.email}</span>
            </p>
            <label htmlFor="signup-otp" className="block">
              <span className="text-sm font-medium text-text">Verification code</span>
              <input
                id="signup-otp"
                name="otp"
                type="number"
                inputMode="numeric"
                autoComplete="one-time-code"
                required
                disabled={isSubmitting}
                className="mt-2 h-11 w-full rounded-lg border border-border bg-card px-3 text-sm text-text outline-none transition placeholder:text-secondary focus:border-accent focus:ring-2 focus:ring-accent/25 disabled:opacity-40"
              />
            </label>
          </>
        ) : null}

        <button
          type="submit"
          disabled={isSubmitting}
          className="h-11 w-full rounded-lg bg-accent px-4 text-sm font-semibold text-text transition hover:brightness-110 disabled:opacity-40"
        >
          {isSubmitting
            ? isOtpSent
              ? "Verifying..."
              : "Sending code..."
            : isOtpSent
              ? "Verify and sign up"
              : "Continue"}
        </button>
      </form>

      <p className="mt-4 text-sm text-secondary">
        Already have an account?{" "}
        <Link href="/login" className="font-semibold text-accent">
          Log in
        </Link>
      </p>
    </AuthCard>
  );
}

function getFormError(caughtError: unknown) {
  if (caughtError instanceof ApiError) {
    return caughtError.message;
  }

  return "Unable to create your account. Please try again.";
}
