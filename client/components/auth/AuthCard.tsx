"use client";

import { useRouter } from "next/navigation";
import { useEffect } from "react";

export function AuthCard({
  eyebrow,
  title,
  children,
}: {
  eyebrow: string;
  title: string;
  children: React.ReactNode;
}) {
  const router = useRouter();

  useEffect(() => {
    const previousOverflow = document.body.style.overflow;
    document.body.style.overflow = "hidden";

    return () => {
      document.body.style.overflow = previousOverflow;
    };
  }, []);

  function handleClose() {
    if (window.history.length > 1) {
      router.back();
      return;
    }

    router.push("/");
  }

  return (
    <section
      className="fixed inset-0 z-[60] flex items-center justify-center bg-background/80 px-4 py-6 backdrop-blur-sm sm:px-6"
      onClick={handleClose}
    >
      <div
        className="relative w-full max-w-md rounded-2xl border border-border bg-surface p-6 shadow-2xl shadow-black/30"
        onClick={(event) => event.stopPropagation()}
      >
        <button
          type="button"
          onClick={handleClose}
          className="absolute right-4 top-4 rounded-full border border-border bg-card px-3 py-1 text-xs font-semibold text-secondary transition hover:text-text focus:outline-none focus:ring-2 focus:ring-accent focus:ring-offset-2 focus:ring-offset-surface"
        >
          Close
        </button>

        <p className="pr-16 text-sm text-secondary">{eyebrow}</p>
        <h1 className="mt-2 pr-16 text-3xl font-bold text-text">{title}</h1>
        {children}
      </div>
    </section>
  );
}
