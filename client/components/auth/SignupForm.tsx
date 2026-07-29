"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useState } from "react";
import { AuthCard } from "@/components/auth/AuthCard";
import { FormField } from "@/components/auth/FormField";
import { signUp } from "@/lib/auth/auth-api";
import { ApiError } from "@/lib/api/http";

export function SignupForm() {
  const router = useRouter();
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  async function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setError(null);
    setIsSubmitting(true);

    const formData = new FormData(event.currentTarget);
    const password = String(formData.get("password"));
    const confirmPassword = String(formData.get("confirmPassword"));

    if (password !== confirmPassword) {
      setError("Passwords do not match.");
      setIsSubmitting(false);
      return;
    }

    try {
      await signUp({
        name: String(formData.get("name")),
        email: String(formData.get("email")),
        password,
        confirmPassword,
      });

      router.push("/login");
    } catch (caughtError) {
      setError(getFormError(caughtError));
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <AuthCard eyebrow="Create an account" title="Sign up">
      <form className="mt-6 space-y-4" onSubmit={handleSubmit}>
        <FormField id="signup-name" label="Name" name="name" autoComplete="name" />
        <FormField
          id="signup-email"
          label="Email"
          name="email"
          type="email"
          autoComplete="email"
        />
        <FormField
          id="signup-password"
          label="Password"
          name="password"
          type="password"
          autoComplete="new-password"
          minLength={8}
        />
        <FormField
          id="signup-confirm-password"
          label="Confirm password"
          name="confirmPassword"
          type="password"
          autoComplete="new-password"
          minLength={8}
        />

        {error ? <p className="text-sm text-sell">{error}</p> : null}

        <button
          type="submit"
          disabled={isSubmitting}
          className="h-11 w-full rounded-lg bg-accent px-4 text-sm font-semibold text-text transition hover:brightness-110 disabled:opacity-40"
        >
          {isSubmitting ? "Creating account..." : "Sign up"}
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
