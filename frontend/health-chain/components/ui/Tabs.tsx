"use client";

import React, { createContext, useContext, useState } from "react";
import { cn } from "@/lib/utils/cn";

export interface TabItem {
  key: string;
  label: string;
  content: React.ReactNode;
}

export interface TabsProps {
  /** Declarative API: pass an array of tabs. */
  items?: TabItem[];
  defaultKey?: string;
  /** Compound API (shadcn-style): pass <TabsList>/<TabsContent> children. */
  defaultValue?: string;
  value?: string;
  onValueChange?: (v: string) => void;
  className?: string;
  children?: React.ReactNode;
}

type Ctx = { value: string; setValue: (v: string) => void };
const TabsCtx = createContext<Ctx | null>(null);
const useTabsCtx = () => {
  const c = useContext(TabsCtx);
  if (!c) throw new Error("TabsList/TabsTrigger/TabsContent need a <Tabs> parent");
  return c;
};

export function Tabs({
  items,
  defaultKey,
  defaultValue,
  value,
  onValueChange,
  className,
  children,
}: TabsProps) {
  const [internal, setInternal] = useState(
    defaultValue ?? defaultKey ?? items?.[0]?.key ?? ""
  );
  const active = value ?? internal;
  const setValue = (v: string) => {
    setInternal(v);
    onValueChange?.(v);
  };

  // Compound API
  if (!items) {
    return (
      <TabsCtx.Provider value={{ value: active, setValue }}>
        <div className={cn("w-full", className)}>{children}</div>
      </TabsCtx.Provider>
    );
  }

  // Declarative API
  const handleKeyDown = (e: React.KeyboardEvent, idx: number) => {
    if (e.key === "ArrowRight")
      setValue(items[(idx + 1) % items.length].key);
    else if (e.key === "ArrowLeft")
      setValue(items[(idx - 1 + items.length) % items.length].key);
  };

  return (
    <div className={cn("w-full", className)}>
      <div role="tablist" className="flex border-b border-border-muted">
        {items.map((item, idx) => (
          <button
            key={item.key}
            role="tab"
            aria-selected={active === item.key}
            aria-controls={`tabpanel-${item.key}`}
            id={`tab-${item.key}`}
            tabIndex={active === item.key ? 0 : -1}
            onClick={() => setValue(item.key)}
            onKeyDown={(e) => handleKeyDown(e, idx)}
            className={cn(
              "px-4 py-2 text-sm font-medium border-b-2 -mb-px transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-[#D32F2F]",
              active === item.key
                ? "border-[#D32F2F] text-[#D32F2F]"
                : "border-transparent text-text-muted hover:text-text-primary"
            )}
          >
            {item.label}
          </button>
        ))}
      </div>
      {items.map((item) => (
        <div
          key={item.key}
          role="tabpanel"
          id={`tabpanel-${item.key}`}
          aria-labelledby={`tab-${item.key}`}
          hidden={active !== item.key}
          className="pt-4"
        >
          {item.content}
        </div>
      ))}
    </div>
  );
}

export function TabsList({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      role="tablist"
      className={cn(
        "inline-flex items-center justify-center gap-1 rounded-lg bg-surface-raised p-1 text-text-muted",
        className
      )}
      {...props}
    />
  );
}

export function TabsTrigger({
  value,
  className,
  ...props
}: React.ButtonHTMLAttributes<HTMLButtonElement> & { value: string }) {
  const { value: active, setValue } = useTabsCtx();
  const selected = active === value;
  return (
    <button
      type="button"
      role="tab"
      aria-selected={selected}
      onClick={() => setValue(value)}
      className={cn(
        "inline-flex items-center justify-center whitespace-nowrap rounded-md px-3 py-1.5 text-sm font-medium transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#D32F2F]",
        selected
          ? "bg-surface text-text-primary shadow-sm"
          : "hover:text-text-primary",
        className
      )}
      {...props}
    />
  );
}

export function TabsContent({
  value,
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement> & { value: string }) {
  const { value: active } = useTabsCtx();
  if (active !== value) return null;
  return <div role="tabpanel" className={cn("mt-2", className)} {...props} />;
}
