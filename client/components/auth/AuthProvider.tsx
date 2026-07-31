"use client";

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
} from "react";
import { refreshSession } from "@/lib/auth/auth-api";
import { decodeAccessToken, isAccessTokenExpired } from "@/lib/auth/token";
import {
  clearStoredAccessToken,
  getStoredAccessToken,
  storeAccessToken,
} from "@/lib/auth/token-storage";
import { getProfile } from "@/lib/profile/profile-api";
import type { Profile } from "@/lib/profile/types";

type AuthContextValue = {
  accessToken: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  isProfileLoading: boolean;
  profile: Profile | null;
  userId: string | null;
  setSession: (accessToken: string) => void;
  clearSession: () => void;
  setProfileCache: (profile: Profile | null) => void;
};

const AuthContext = createContext<AuthContextValue | null>(null);

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [accessToken, setAccessToken] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [profile, setProfile] = useState<Profile | null>(null);
  const [isProfileLoading, setIsProfileLoading] = useState(true);

  const clearSession = useCallback(() => {
    clearStoredAccessToken();
    setAccessToken(null);
    setProfile(null);
    setIsProfileLoading(false);
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

  useEffect(() => {
    if (isLoading) {
      return;
    }

    if (!accessToken) {
      // eslint-disable-next-line react-hooks/set-state-in-effect
      setProfile(null);
      setIsProfileLoading(false);
      return;
    }

    let isCurrent = true;
    setIsProfileLoading(true);

    getProfile()
      .then((profileResponse) => {
        if (isCurrent) {
          setProfile(profileResponse);
        }
      })
      .catch(() => {
        if (isCurrent) {
          setProfile(null);
        }
      })
      .finally(() => {
        if (isCurrent) {
          setIsProfileLoading(false);
        }
      });

    return () => {
      isCurrent = false;
    };
  }, [accessToken, isLoading]);

  const setProfileCache = useCallback((nextProfile: Profile | null) => {
    setProfile(nextProfile);
  }, []);

  const value = useMemo<AuthContextValue>(() => {
    const tokenPayload = accessToken ? decodeAccessToken(accessToken) : null;

    return {
      accessToken,
      isAuthenticated: Boolean(accessToken),
      isLoading,
      isProfileLoading,
      profile,
      userId: tokenPayload?.sub ?? null,
      setSession,
      clearSession,
      setProfileCache,
    };
  }, [
    accessToken,
    clearSession,
    isLoading,
    isProfileLoading,
    profile,
    setProfileCache,
    setSession,
  ]);

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth() {
  const context = useContext(AuthContext);

  if (!context) {
    throw new Error("useAuth must be used within AuthProvider");
  }

  return context;
}
