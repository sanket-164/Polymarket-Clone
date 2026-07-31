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
    <header className="sticky top-0 z-50 border-b border-border bg-surface/80 backdrop-blur-md">
      <nav
        className="mx-auto flex w-full max-w-7xl flex-col gap-4 px-4 py-3 sm:px-6 lg:flex-row lg:items-center lg:gap-6 lg:px-8"
        aria-label="Primary"
      >
        <div className="flex items-center justify-between gap-4">
          <Link
            href="/"
            className="group flex shrink-0 items-center gap-2.5 rounded-xl transition focus:outline-none focus:ring-2 focus:ring-accent focus:ring-offset-2 focus:ring-offset-surface"
          >
            <span className="flex size-9 items-center justify-center rounded-xl border border-border bg-card text-sm font-bold text-accent shadow-sm transition group-hover:border-accent/50 group-hover:bg-accent/5">
              PM
            </span>
            <span className="text-lg font-bold tracking-tight text-text sm:text-xl">
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
          <div className="relative">
            <svg
              className="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-secondary"
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
              strokeWidth={2}
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                d="m21 21-5.197-5.197m0 0A7.5 7.5 0 1 0 5.196 5.196a7.5 7.5 0 0 0 10.607 10.607Z"
              />
            </svg>
            <input
              id="market-search"
              name="q"
              type="search"
              placeholder="Search markets..."
              className="h-10 w-full rounded-lg border border-border bg-card pl-9 pr-4 text-sm text-text outline-none transition placeholder:text-secondary focus:border-accent focus:ring-2 focus:ring-accent/20"
            />
          </div>
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
      <div className="h-10 w-24 animate-pulse rounded-lg border border-border bg-card" />
    );
  }

  if (isAuthenticated) {
    return (
      <div className="flex items-center gap-2">
        <Link
          href="/holdings"
          className="inline-flex items-center justify-center rounded-lg border border-transparent px-3 py-2 text-sm font-medium text-secondary transition-all duration-200 hover:border-border hover:bg-card hover:text-text focus:outline-none focus:ring-2 focus:ring-accent focus:ring-offset-2 focus:ring-offset-surface"
        >
          <svg
            className="mr-2 size-4"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            strokeWidth={2}
            stroke="currentColor"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              d="M2.25 18 9 11.25l4.306 4.306a11.95 11.95 0 0 1 5.814-5.518l2.74-1.22m0 0-5.94-2.281m5.94 2.28-2.28 5.941"
            />
          </svg>
          Holdings
        </Link>
        <Link
          href="/orders"
          className="inline-flex items-center justify-center rounded-lg border border-transparent px-3 py-2 text-sm font-medium text-secondary transition-all duration-200 hover:border-border hover:bg-card hover:text-text focus:outline-none focus:ring-2 focus:ring-accent focus:ring-offset-2 focus:ring-offset-surface"
        >
          <svg
            className="mr-2 size-4"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            strokeWidth={2}
            stroke="currentColor"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              d="M9 12h3.75M9 15h3.75M9 18h3.75m3 .75H18a2.25 2.25 0 0 0 2.25-2.25V6.108c0-1.135-.845-2.098-1.976-2.192a48.424 48.424 0 0 0-1.123-.08m-5.801 0c-.065.21-.1.433-.1.664 0 .414.336.75.75.75h4.5a.75.75 0 0 0 .75-.75 2.25 2.25 0 0 0-.1-.664m-5.8 0A2.251 2.251 0 0 1 13.5 2.25H15c1.012 0 1.867.668 2.15 1.586m-5.8 0c-.376.023-.75.05-1.124.08C9.095 4.01 8.25 4.973 8.25 6.108V8.25m0 0H4.875c-.621 0-1.125.504-1.125 1.125v11.25c0 .621.504 1.125 1.125 1.125h9.75c.621 0 1.125-.504 1.125-1.125V9.375c0-.621-.504-1.125-1.125-1.125H8.25ZM6.75 12h.008v.008H6.75V12Zm0 3h.008v.008H6.75V15Zm0 3h.008v.008H6.75V18Z"
            />
          </svg>
          Orders
        </Link>
        <div className="ml-1 h-6 w-px bg-border" />
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
        className="inline-flex items-center justify-center rounded-lg border border-border bg-card px-4 py-2 text-sm font-semibold text-text transition-all duration-200 hover:border-accent/50 hover:bg-accent/5 hover:text-accent focus:outline-none focus:ring-2 focus:ring-accent focus:ring-offset-2 focus:ring-offset-surface"
      >
        Log in
      </Link>
      <Link
        href="/signup"
        className="inline-flex items-center justify-center rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-text shadow-sm shadow-accent/20 transition-all duration-200 hover:brightness-110 hover:shadow-md hover:shadow-accent/30 focus:outline-none focus:ring-2 focus:ring-accent focus:ring-offset-2 focus:ring-offset-surface"
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
        className="group flex items-center gap-2 rounded-full border border-border bg-card p-1 pr-3 transition-all duration-200 hover:border-accent/50 hover:shadow-md focus:outline-none focus:ring-2 focus:ring-accent focus:ring-offset-2 focus:ring-offset-surface"
        aria-label="Open profile menu"
        aria-haspopup="menu"
        aria-expanded={isOpen}
      >
        <Image
          src={avatarSrc}
          alt={profile?.name ? `${profile.name} profile` : "Profile"}
          width={32}
          height={32}
          unoptimized={Boolean(profile?.picture)}
          className="size-8 rounded-full object-cover ring-2 ring-transparent transition group-hover:ring-accent/20"
        />
        <svg
          className={`size-4 text-secondary transition-transform duration-200 ${isOpen ? "rotate-180" : ""}`}
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 24 24"
          strokeWidth={2}
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            d="m19.5 8.25-7.5 7.5-7.5-7.5"
          />
        </svg>
      </button>

      {isOpen && (
        <div
          role="menu"
          aria-label="Profile actions"
          className="absolute right-0 top-full z-50 mt-2 w-56 origin-top-right rounded-xl border border-border bg-card p-1.5 shadow-xl shadow-black/5 animate-in fade-in slide-in-from-top-2 duration-200"
        >
          <div className="px-3 py-2.5">
            <p className="text-sm font-semibold text-text">
              {profile?.name ?? "Your account"}
            </p>
            <p className="mt-0.5 truncate text-xs text-secondary">
              {isProfileLoading
                ? "Loading profile..."
                : (profile?.email ?? "Manage your settings")}
            </p>
          </div>

          <div className="my-1.5 h-px bg-border" />

          <Link
            href="/profile"
            role="menuitem"
            onClick={() => setIsOpen(false)}
            className="flex w-full items-center rounded-lg px-3 py-2 text-sm font-medium text-text transition-colors hover:bg-accent/10 hover:text-accent focus:outline-none focus:bg-accent/10"
          >
            <svg
              className="mr-2.5 size-4"
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
              strokeWidth={2}
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                d="M15.75 6a3.75 3.75 0 1 1-7.5 0 3.75 3.75 0 0 1 7.5 0ZM4.501 20.118a7.5 7.5 0 0 1 14.998 0A17.933 17.933 0 0 1 12 21.75c-2.676 0-5.216-.584-7.499-1.632Z"
              />
            </svg>
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
            className="mt-0.5 flex w-full items-center rounded-lg px-3 py-2 text-sm font-medium text-sell transition-colors hover:bg-sell/10 disabled:opacity-50"
          >
            <svg
              className="mr-2.5 size-4"
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
              strokeWidth={2}
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                d="M15.75 9V5.25A2.25 2.25 0 0 0 13.5 3h-6a2.25 2.25 0 0 0-2.25 2.25v13.5A2.25 2.25 0 0 0 7.5 21h6a2.25 2.25 0 0 0 2.25-2.25V15m3 0 3-3m0 0-3-3m3 3H9"
              />
            </svg>
            {isLoggingOut ? "Logging out..." : "Log out"}
          </button>
        </div>
      )}
    </div>
  );
}
