"use client";

import Image from "next/image";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { useEffect, useRef, useState } from "react";
import { useAuth } from "@/components/auth/AuthProvider";
import { logout } from "@/lib/auth/auth-api";
import type { Profile } from "@/lib/profile/types";

export function Navbar() {
  const {
    isAuthenticated,
    isLoading,
    isProfileLoading,
    profile,
    clearSession,
  } = useAuth();
  const router = useRouter();
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
              isProfileLoading={isProfileLoading}
              profile={profile}
              isLoggingOut={isLoggingOut}
              onLogout={handleLogout}
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
            isProfileLoading={isProfileLoading}
            profile={profile}
            isLoggingOut={isLoggingOut}
            onLogout={handleLogout}
          />
        </div>
      </nav>
    </header>
  );
}

type NavbarActionsProps = {
  isAuthenticated: boolean;
  isLoading: boolean;
  isProfileLoading: boolean;
  profile: Profile | null;
  isLoggingOut: boolean;
  onLogout: () => Promise<void>;
};

function NavbarActions({
  isAuthenticated,
  isLoading,
  isProfileLoading,
  profile,
  isLoggingOut,
  onLogout,
}: NavbarActionsProps) {
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
        <ProfileMenu
          profile={profile}
          isProfileLoading={isProfileLoading}
          isLoggingOut={isLoggingOut}
          onLogout={onLogout}
        />
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

type ProfileMenuProps = {
  profile: Profile | null;
  isProfileLoading: boolean;
  isLoggingOut: boolean;
  onLogout: () => Promise<void>;
};

function ProfileMenu({
  profile,
  isProfileLoading,
  isLoggingOut,
  onLogout,
}: ProfileMenuProps) {
  const [isOpen, setIsOpen] = useState(false);
  const menuRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    function handleDocumentClick(event: MouseEvent) {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    }

    function handleEscape(event: KeyboardEvent) {
      if (event.key === "Escape") {
        setIsOpen(false);
      }
    }

    document.addEventListener("mousedown", handleDocumentClick);
    document.addEventListener("keydown", handleEscape);

    return () => {
      document.removeEventListener("mousedown", handleDocumentClick);
      document.removeEventListener("keydown", handleEscape);
    };
  }, []);

  const avatarSrc = profile?.picture || "/default-profile.svg";

  return (
    <div ref={menuRef} className="relative">
      <button
        type="button"
        onClick={() => setIsOpen((current) => !current)}
        className="rounded-full border border-border bg-card p-1 transition hover:border-accent focus:outline-none focus:ring-2 focus:ring-accent focus:ring-offset-2 focus:ring-offset-surface"
        aria-label="Open profile menu"
        aria-haspopup="menu"
        aria-expanded={isOpen}
      >
        <Image
          src={avatarSrc}
          alt={profile?.name ? `${profile.name} profile` : "Profile"}
          width={36}
          height={36}
          unoptimized={Boolean(profile?.picture)}
          className="size-9 rounded-full object-cover"
        />
      </button>

      {isOpen ? (
        <div
          role="menu"
          aria-label="Profile actions"
          className="absolute right-0 top-full z-50 mt-3 w-56 rounded-xl border border-border bg-card p-2"
        >
          <div className="px-3 py-2">
            <p className="text-sm font-semibold text-text">
              {profile?.name ?? "Your account"}
            </p>
            <p className="mt-1 text-xs text-secondary">
              {isProfileLoading
                ? "Loading profile..."
                : (profile?.email ?? "Profile settings")}
            </p>
          </div>

          <Link
            href="/profile"
            role="menuitem"
            onClick={() => setIsOpen(false)}
            className="flex w-full items-center rounded-lg px-3 py-2 text-sm font-semibold text-text transition hover:bg-surface focus:outline-none focus:bg-surface"
          >
            View profile
          </Link>

          <button
            type="button"
            role="menuitem"
            onClick={() => {
              setIsOpen(false);
              void onLogout();
            }}
            disabled={isLoggingOut}
            className="mt-1 flex w-full items-center rounded-lg px-3 py-2 text-sm font-semibold text-sell transition hover:bg-surface disabled:opacity-40"
          >
            {isLoggingOut ? "Logging out..." : "Log out"}
          </button>
        </div>
      ) : null}
    </div>
  );
}
