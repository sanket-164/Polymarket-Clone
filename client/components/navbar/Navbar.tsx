"use client";

import Image from "next/image";
import Link from "next/link";
import { useAuth } from "@/components/auth/AuthProvider";

export function Navbar() {
  const { isAuthenticated, isLoading } = useAuth();

  return (
    <header className="sticky top-0 z-50 border-b border-border bg-surface">
      <nav
        className="mx-auto flex w-full max-w-7xl flex-col gap-3 px-4 py-3 sm:px-6 lg:flex-row lg:items-center lg:gap-6 lg:px-8"
        aria-label="Primary"
      >
        <div className="flex items-center justify-between gap-4">
          <Link
            href="/"
            className="flex shrink-0 items-center gap-2 rounded-lg text-text transition hover:text-accent focus:outline-none focus:ring-2 focus:ring-accent focus:ring-offset-2 focus:ring-offset-surface"
          >
            <span className="flex size-9 items-center justify-center rounded-xl border border-border bg-card text-sm font-bold text-accent">
              PM
            </span>
            <span className="text-base font-bold tracking-normal sm:text-lg">
              Polymarket
            </span>
          </Link>

          <div className="flex shrink-0 items-center gap-2 lg:hidden">
            <NavbarActions
              isAuthenticated={isAuthenticated}
              isLoading={isLoading}
            />
          </div>
        </div>

        <form className="min-w-0 flex-1" role="search">
          <label htmlFor="market-search" className="sr-only">
            Search markets
          </label>
          <input
            id="market-search"
            name="q"
            type="search"
            placeholder="Search markets"
            className="h-11 w-full rounded-lg border border-border bg-card px-4 text-sm text-text outline-none transition placeholder:text-secondary focus:border-accent focus:ring-2 focus:ring-accent/25"
          />
        </form>

        <div className="hidden shrink-0 items-center gap-2 lg:flex">
          <NavbarActions
            isAuthenticated={isAuthenticated}
            isLoading={isLoading}
          />
        </div>
      </nav>
    </header>
  );
}

type NavbarActionsProps = {
  isAuthenticated: boolean;
  isLoading: boolean;
};

function NavbarActions({ isAuthenticated, isLoading }: NavbarActionsProps) {
  if (isLoading) {
    return (
      <div className="h-10 w-24 rounded-lg border border-border bg-card" />
    );
  }

  if (isAuthenticated) {
    return (
      <div className="flex items-center gap-2">
        <Link
          href="/holdings"
          className="rounded-lg border border-border bg-surface px-3 py-2 text-sm font-semibold text-text transition hover:bg-card focus:outline-none focus:ring-2 focus:ring-accent focus:ring-offset-2 focus:ring-offset-surface"
        >
          Holdings
        </Link>
        <Link
          href="/orders"
          className="rounded-lg border border-border bg-surface px-3 py-2 text-sm font-semibold text-text transition hover:bg-card focus:outline-none focus:ring-2 focus:ring-accent focus:ring-offset-2 focus:ring-offset-surface"
        >
          Orders
        </Link>
        <Link
          href="/profile"
          className="rounded-full border border-border bg-card p-1 transition hover:border-accent focus:outline-none focus:ring-2 focus:ring-accent focus:ring-offset-2 focus:ring-offset-surface"
          aria-label="Open profile"
        >
          <Image
            src="/default-profile.svg"
            alt="Profile"
            width={36}
            height={36}
            className="size-9 rounded-full object-cover"
          />
        </Link>
      </div>
    );
  }

  return (
    <>
      <Link
        href="/login"
        className="rounded-lg border border-border bg-surface px-3 py-2 text-sm font-semibold text-text transition hover:bg-card focus:outline-none focus:ring-2 focus:ring-accent focus:ring-offset-2 focus:ring-offset-surface sm:px-4"
      >
        Log in
      </Link>
      <Link
        href="/signup"
        className="rounded-lg bg-accent px-3 py-2 text-sm font-semibold text-text transition hover:brightness-110 focus:outline-none focus:ring-2 focus:ring-accent focus:ring-offset-2 focus:ring-offset-surface sm:px-4"
      >
        Sign up
      </Link>
    </>
  );
}
