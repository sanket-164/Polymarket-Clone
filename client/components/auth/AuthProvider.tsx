"use client";

import { createContext, useCallback, useContext, useEffect, useMemo, useState } from "react";
import { refreshSession } from "@/lib/auth/auth-api";
import { clearStoredAccessToken, getStoredAccessToken, storeAccessToken } from "@/lib/auth/token-storage";
import { decodeAccessToken, isAccessTokenExpired } from "@/lib/auth/token";

type AuthContextValue = {
  accessToken: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  userId: string | null;
  setSession: (accessToken: string) => void;
  clearSession: () => void;
};

const AuthContext = createContext<AuthContextValue | null>(null);

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [accessToken, setAccessToken] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  const clearSession = useCallback(() => {
    clearStoredAccessToken();
    setAccessToken(null);
  }, []);

  const setSession = useCallback((token: string) => {
    storeAccessToken(token);
    setAccessToken(token);
  }, []);

  useEffect(() => {
    async function loadSession() {
      const storedToken = getStoredAccessToken();

      if (storedToken && !isAccessTokenExpired(storedToken)) {
        setAccessToken(storedToken);
        setIsLoading(false);
        return;
      }

      try {
        const refreshedSession = await refreshSession();
        setSession(refreshedSession.access_token);
      } catch {
        clearSession();
      } finally {
        setIsLoading(false);
      }
    }

    void loadSession();
  }, [clearSession, setSession]);

  const value = useMemo<AuthContextValue>(() => {
    const tokenPayload = accessToken ? decodeAccessToken(accessToken) : null;

    return {
      accessToken,
      isAuthenticated: Boolean(accessToken),
      isLoading,
      userId: tokenPayload?.sub ?? null,
      setSession,
      clearSession,
    };
  }, [accessToken, clearSession, isLoading, setSession]);

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth() {
  const context = useContext(AuthContext);

  if (!context) {
    throw new Error("useAuth must be used within AuthProvider");
  }

  return context;
}
