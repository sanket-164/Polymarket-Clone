"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useState } from "react";
import { useAuth } from "@/components/auth/AuthProvider";
import { AuthCard } from "@/components/auth/AuthCard";
import { FormField } from "@/components/auth/FormField";
import { signIn } from "@/lib/auth/auth-api";
import { ApiError } from "@/lib/api/http";

export function LoginForm() {
  const router = useRouter();
  const { setSession } = useAuth();
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  async function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setError(null);
    setIsSubmitting(true);

    const formData = new FormData(event.currentTarget);

    try {
      const session = await signIn({
        email: String(formData.get("email")),
        password: String(formData.get("password")),
      });

      setSession(session.access_token);
      router.push("/profile");
    } catch (caughtError) {
      setError(getFormError(caughtError));
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <AuthCard eyebrow="Welcome back" title="Log in">
      <form className="mt-6 space-y-4" onSubmit={handleSubmit}>
        <FormField
          id="login-email"
          label="Email"
          name="email"
          type="email"
          autoComplete="email"
        />
        <FormField
          id="login-password"
          label="Password"
          name="password"
          type="password"
          autoComplete="current-password"
          minLength={8}
        />

        {error ? <p className="text-sm text-sell">{error}</p> : null}

        <button
          type="submit"
          disabled={isSubmitting}
          className="h-11 w-full rounded-lg bg-accent px-4 text-sm font-semibold text-text transition hover:brightness-110 disabled:opacity-40"
        >
          {isSubmitting ? "Logging in..." : "Log in"}
        </button>
      </form>

      <div className="mt-4 flex items-center justify-between gap-4 text-sm">
        <Link href="/reset-password" className="text-secondary transition hover:text-accent">
          Reset password
        </Link>
        <Link href="/signup" className="font-semibold text-accent">
          Sign up
        </Link>
      </div>
    </AuthCard>
  );
}

function getFormError(caughtError: unknown) {
  if (caughtError instanceof ApiError) {
    return caughtError.message;
  }

  return "Unable to log in. Please try again.";
}
