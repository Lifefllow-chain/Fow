"use client";

/**
 * Compatibility shim: some pages import `useAuth` from
 * `@/components/providers/auth-provider`. The real auth logic lives in
 * `@/lib/hooks/useAuth` (backed by the Zustand `auth.store`). This re-exports
 * it and provides a no-op `AuthProvider` so either import path works.
 */

import React from "react";
import { useAuth as useAuthHook } from "@/lib/hooks/useAuth";

export const useAuth = useAuthHook;

export function AuthProvider({ children }: { children: React.ReactNode }) {
  return <>{children}</>;
}

export default AuthProvider;
