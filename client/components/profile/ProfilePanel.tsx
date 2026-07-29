"use client";

import Image from "next/image";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { useState } from "react";
import { useAuth } from "@/components/auth/AuthProvider";
import { logout } from "@/lib/auth/auth-api";

export function ProfilePanel() {
  const router = useRouter();
  const { clearSession, isAuthenticated, isLoading, userId } = useAuth();
  const [isLoggingOut, setIsLoggingOut] = useState(false);

  async function handleLogout() {
    setIsLoggingOut(true);

    try {
      await logout();
    } finally {
      clearSession();
      router.push("/login");
    }
  }

  if (isLoading) {
    return <ProfileShell title="Loading profile" description="Checking your session..." />;
  }

  if (!isAuthenticated) {
    return (
      <ProfileShell
        title="Log in required"
        description="You need an active session to view your profile."
      />
    );
  }

  return (
    <section className="mx-auto w-full max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
      <div className="rounded-2xl border border-border bg-surface p-6">
        <div className="flex flex-col gap-6 sm:flex-row sm:items-center sm:justify-between">
          <div className="flex items-center gap-4">
            <Image
              src="/default-profile.svg"
              alt="Profile"
              width={56}
              height={56}
              className="size-14 rounded-full object-cover"
            />
            <div>
              <p className="text-sm text-secondary">Profile</p>
              <h1 className="mt-1 text-3xl font-bold text-text">Your account</h1>
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

        <dl className="mt-6 grid gap-3 rounded-xl border border-border bg-card p-4 sm:grid-cols-2">
          <div>
            <dt className="text-xs uppercase text-secondary">User ID</dt>
            <dd className="mt-1 break-all font-mono text-sm text-text">{userId ?? "Unavailable"}</dd>
          </div>
          <div>
            <dt className="text-xs uppercase text-secondary">Session</dt>
            <dd className="mt-1 text-sm font-semibold text-buy">Active</dd>
          </div>
        </dl>
      </div>
    </section>
  );
}

function ProfileShell({ title, description }: { title: string; description: string }) {
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
