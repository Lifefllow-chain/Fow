"use client";

import { useRef, useState } from "react";
import gsap from "gsap";
import { useGSAP } from "@gsap/react";
import {
  ShieldCheck,
  Droplet,
  Microscope,
  Snowflake,
  Truck,
  HeartPulse,
  Search,
  Link2,
} from "lucide-react";

type Event = {
  step: string;
  icon: React.ComponentType<{ size?: number; strokeWidth?: number; className?: string }>;
  actor: string;
  detail: string;
  time: string;
  hash: string;
  block: number;
};

type Unit = {
  id: string;
  bloodGroup: string;
  status: "safe" | "quarantined";
  summary: string;
  events: Event[];
};

const DB: Record<string, Unit> = {
  "SB-2K7F-9410": {
    id: "SB-2K7F-9410",
    bloodGroup: "O+",
    status: "safe",
    summary: "All screens negative · cold chain intact · delivered sealed",
    events: [
      { step: "Donation", icon: Droplet, actor: "Donor D-4471 · Collector C-12", detail: "Whole blood, 450 mL — Lagos drive #88", time: "2025-08-10 · 09:14", hash: "0x9af1a3c20e77b210", block: 4820113 },
      { step: "Screening", icon: Microscope, actor: "Lab tech L-08", detail: "HIV · HBV · HCV · Syphilis — all non-reactive", time: "2025-08-10 · 15:02", hash: "0x3c7d0091f581abcd", block: 4820551 },
      { step: "Storage", icon: Snowflake, actor: "Bank BK-Lagos-01", detail: "Held at 2–6°C, continuous temp log, 41 h", time: "2025-08-10 → 08-12", hash: "0x1e55b7cc4f9c1122", block: 4821004 },
      { step: "Delivery", icon: Truck, actor: "Rider R-19", detail: "Sealed, temp-logged transfer to St. Mary's", time: "2025-08-12 · 14:20", hash: "0xa02867d3170099ee", block: 4822610 },
      { step: "Transfusion", icon: HeartPulse, actor: "Ward 3 · Nurse N-27", detail: "Patient cross-match verified at bedside", time: "2025-08-12 · 16:05", hash: "0x77b2e6a44513cdef", block: 4822744 },
    ],
  },
  "SB-9QX1-2203": {
    id: "SB-9QX1-2203",
    bloodGroup: "A-",
    status: "quarantined",
    summary: "Cold-chain excursion during transit — held for review",
    events: [
      { step: "Donation", icon: Droplet, actor: "Donor D-1180 · Collector C-04", detail: "Whole blood, 470 mL — Abuja drive #21", time: "2025-08-18 · 10:40", hash: "0x55aa11bb22cc33dd", block: 4890220 },
      { step: "Screening", icon: Microscope, actor: "Lab tech L-02", detail: "HIV · HBV · HCV · Syphilis — all non-reactive", time: "2025-08-18 · 17:22", hash: "0x66bb77cc88dd99ee", block: 4890788 },
      { step: "Storage", icon: Snowflake, actor: "Bank BK-Abuja-02", detail: "Held at 2–6°C, 26 h", time: "2025-08-18 → 08-19", hash: "0x99ee00ff11002233", block: 4891140 },
      { step: "Delivery", icon: Truck, actor: "Rider R-07", detail: "Excursion: 11.4°C for 38 min — flag raised on-chain", time: "2025-08-19 · 13:05", hash: "0xdeadbeef00c0ffee", block: 4892001 },
    ],
  },
};

const SAMPLE_IDS = Object.keys(DB);

export default function SmartBagTracker() {
  const [query, setQuery] = useState("SB-2K7F-9410");
  const [unit, setUnit] = useState<Unit | null>(DB["SB-2K7F-9410"]);
  const [error, setError] = useState<string | null>(null);
  const scope = useRef<HTMLDivElement>(null);

  const lookup = (raw: string) => {
    const key = raw.trim().toUpperCase();
    const found = DB[key];
    if (!found) {
      setUnit(null);
      setError(`No unit found for "${key}". Try one of the sample IDs below.`);
      return;
    }
    setError(null);
    setUnit(found);
  };

  useGSAP(
    () => {
      if (!unit) return;
      if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;

      gsap.set(".tk-summary", { y: 20, opacity: 0 });
      gsap.set(".tk-row", { x: 28, opacity: 0 });

      const tl = gsap.timeline({ defaults: { ease: "power3.out" } });
      tl.to(".tk-summary", { y: 0, opacity: 1, duration: 0.5 })
        .to(
          ".tk-row",
          { x: 0, opacity: 1, duration: 0.5, stagger: 0.14 },
          "-=0.1"
        );
    },
    { scope, dependencies: [unit?.id] }
  );

  return (
    <div ref={scope} className="mx-auto max-w-3xl px-6">
      {/* search */}
      <form
        onSubmit={(e) => {
          e.preventDefault();
          lookup(query);
        }}
        className="flex flex-col gap-3 sm:flex-row"
      >
        <div className="flex flex-1 items-center gap-3 rounded-xl border border-border-muted bg-surface px-4 shadow-card focus-within:border-oxblood-500">
          <Search size={18} className="text-text-muted" />
          <input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Enter a SmartBag ID — e.g. SB-2K7F-9410"
            aria-label="SmartBag ID"
            className="h-12 w-full border-none bg-transparent font-mono text-[14px] text-text-primary placeholder-text-muted focus:outline-none"
          />
        </div>
        <button
          type="submit"
          className="inline-flex h-12 items-center justify-center rounded-xl bg-oxblood-800 px-7 font-roboto text-sm font-semibold text-white transition hover:bg-oxblood-900"
        >
          Verify unit
        </button>
      </form>

      <div className="mt-3 flex flex-wrap items-center gap-2 font-poppins text-[12px] text-text-muted">
        Sample IDs:
        {SAMPLE_IDS.map((id) => (
          <button
            key={id}
            onClick={() => {
              setQuery(id);
              lookup(id);
            }}
            className="rounded-full border border-border-muted px-3 py-1 font-mono text-[11.5px] text-oxblood-700 transition hover:bg-oxblood-50 dark:hover:bg-oxblood-950"
          >
            {id}
          </button>
        ))}
      </div>

      {error && (
        <p className="mt-8 rounded-xl border border-status-critical/30 bg-status-critical/5 p-4 font-roboto text-[14px] text-status-critical">
          {error}
        </p>
      )}

      {unit && (
        <div className="mt-10">
          {/* summary card */}
          <div
            className={`tk-summary rounded-2xl border p-6 shadow-card ${
              unit.status === "safe"
                ? "border-oxblood-200 bg-oxblood-50 dark:border-oxblood-800 dark:bg-oxblood-950"
                : "border-status-warning/40 bg-status-warning/5"
            }`}
          >
            <div className="flex flex-wrap items-start justify-between gap-4">
              <div>
                <p className="font-poppins text-[11px] uppercase tracking-wider text-text-muted">
                  SmartBag
                </p>
                <p className="font-manrope text-2xl font-bold text-text-primary">
                  {unit.id}
                </p>
                <p className="mt-1 font-poppins text-[13px] text-text-muted">
                  Blood group <span className="font-semibold text-text-primary">{unit.bloodGroup}</span>
                </p>
              </div>
              <span
                className={`inline-flex items-center gap-2 rounded-full px-4 py-1.5 font-poppins text-[13px] font-semibold ${
                  unit.status === "safe"
                    ? "bg-oxblood-800 text-white"
                    : "bg-status-warning text-white"
                }`}
              >
                <ShieldCheck size={15} />
                {unit.status === "safe" ? "Verified safe to transfuse" : "Quarantined — under review"}
              </span>
            </div>
            <p className="mt-4 font-roboto text-[14px] leading-relaxed text-text-secondary">
              {unit.summary}
            </p>
          </div>

          {/* chain of custody */}
          <h2 className="mt-10 font-manrope text-lg font-bold text-text-primary">
            Chain of custody
          </h2>
          <p className="mb-6 font-poppins text-[13px] text-text-muted">
            Each event is hashed and anchored on Soroban. The hash of every record
            includes the one before it — so a single edit breaks the chain.
          </p>

          <ol className="relative">
            {unit.events.map((ev, i) => (
              <li key={ev.hash} className="tk-row relative flex gap-5 pb-8 last:pb-0">
                <div className="flex flex-col items-center">
                  <span className="flex h-11 w-11 shrink-0 items-center justify-center rounded-full bg-oxblood-800 text-white">
                    <ev.icon size={18} strokeWidth={1.75} />
                  </span>
                  {i < unit.events.length - 1 && (
                    <span className="tk-line mt-1 w-[2px] flex-1 bg-gradient-to-b from-oxblood-400 to-oxblood-200 dark:to-oxblood-800" />
                  )}
                </div>
                <div className="flex-1 rounded-xl border border-border-muted bg-surface p-4 shadow-card">
                  <div className="flex flex-wrap items-baseline justify-between gap-2">
                    <p className="font-manrope text-[15px] font-bold text-text-primary">
                      {ev.step}
                    </p>
                    <p className="font-poppins text-[12px] text-text-muted">{ev.time}</p>
                  </div>
                  <p className="mt-1 font-poppins text-[13px] text-text-secondary">
                    {ev.actor}
                  </p>
                  <p className="mt-1 font-poppins text-[13px] text-text-muted">
                    {ev.detail}
                  </p>
                  <div className="mt-3 flex flex-wrap items-center gap-x-4 gap-y-1 border-t border-border-muted pt-3 font-mono text-[11.5px] text-oxblood-600 dark:text-oxblood-400">
                    <span className="inline-flex items-center gap-1.5">
                      <Link2 size={12} /> {ev.hash}
                    </span>
                    <span className="text-text-muted">block #{ev.block.toLocaleString()}</span>
                  </div>
                </div>
              </li>
            ))}
          </ol>

          <p className="mt-6 flex items-center gap-2 font-poppins text-[12.5px] text-text-muted">
            <ShieldCheck size={14} className="text-status-resolved" />
            Chain integrity check passed — {unit.events.length} linked records, no gaps.
          </p>
        </div>
      )}
    </div>
  );
}
