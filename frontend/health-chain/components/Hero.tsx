"use client";

import { useRef } from "react";
import Link from "next/link";
import gsap from "gsap";
import { useGSAP } from "@gsap/react";

const CUSTODY = [
  { step: "Donation", who: "Donor D-4471", hash: "0x9af1…c20e" },
  { step: "Screening", who: "Lab tech L-08 · HIV / HBV / HCV / Syphilis", hash: "0x3c7d…81ab" },
  { step: "Storage", who: "Cold chain 2–6°C · Bank BK-Lagos-01", hash: "0x1e55…4f9c" },
  { step: "Delivery", who: "Rider R-19 · sealed, temp-logged", hash: "0xa028…7d31" },
  { step: "Transfusion", who: "Ward 3 · patient match verified", hash: "0x77b2…e6a4" },
];

export default function Hero() {
  const root = useRef<HTMLElement>(null);

  useGSAP(
    () => {
      gsap.to(".hero-drop", {
        y: -14,
        duration: 3,
        ease: "sine.inOut",
        repeat: -1,
        yoyo: true,
      });

      if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;

      // Establish start states, then animate every group to its natural spot.
      // Using set()+to() (rather than from()) keeps the reveal reliable even if
      // React re-renders a node mid-timeline.
      const groups: [string, gsap.TweenVars, number][] = [
        [".hero-eyebrow", { y: 20, opacity: 0 }, 0],
        [".hero-title span", { yPercent: 110, opacity: 0 }, 0.15],
        [".hero-sub", { y: 24, opacity: 0 }, 0.5],
        [".hero-cta", { y: 20, opacity: 0 }, 0.7],
        [".hero-stat", { y: 20, opacity: 0 }, 0.9],
        [".hero-card", { x: 60, opacity: 0 }, 0.35],
        [".hero-row", { x: 26, opacity: 0 }, 0.7],
      ];
      groups.forEach(([sel, vars]) => gsap.set(sel, vars));

      const tl = gsap.timeline({
        defaults: { ease: "power3.out", duration: 0.8 },
      });
      groups.forEach(([sel, , at]) => {
        tl.to(
          sel,
          { x: 0, y: 0, yPercent: 0, opacity: 1, scale: 1, stagger: 0.12 },
          at
        );
      });
    },
    { scope: root }
  );

  return (
    <section
      ref={root}
      className="relative w-full overflow-hidden bg-surface pt-32 pb-20 md:pt-44 md:pb-28"
    >
      {/* ambient background */}
      <div className="pointer-events-none absolute inset-0 bg-oxblood-radial" />
      <div className="pointer-events-none absolute -top-40 -left-40 h-[520px] w-[520px] rounded-full bg-oxblood-100/60 blur-3xl dark:bg-oxblood-900/30" />

      <div className="relative z-10 mx-auto grid max-w-[1288px] grid-cols-1 items-center gap-16 px-6 lg:grid-cols-[1.05fr_0.95fr]">
        {/* Left: copy */}
        <div>
          <span className="hero-eyebrow inline-flex items-center gap-2 rounded-full border border-oxblood-200 bg-oxblood-50 px-4 py-1.5 font-poppins text-[13px] font-medium text-oxblood-700 dark:border-oxblood-800 dark:bg-oxblood-950 dark:text-oxblood-300">
            <span className="relative flex h-2 w-2">
              <span className="absolute inline-flex h-full w-full animate-pulse-ring rounded-full bg-oxblood-500" />
              <span className="relative inline-flex h-2 w-2 rounded-full bg-oxblood-600" />
            </span>
            Built on Stellar · Soroban smart contracts · Open source
          </span>

          <h1 className="hero-title mt-6 font-manrope text-[42px] font-bold leading-[1.05] tracking-tight text-text-primary sm:text-[56px] md:text-[64px]">
            <span className="block overflow-hidden">
              <span className="block">Vein&#8288;-&#8288;to&#8288;-&#8288;vein trust</span>
            </span>
            <span className="block overflow-hidden">
              <span className="block text-gradient-oxblood">for every unit of blood.</span>
            </span>
          </h1>

          <p className="hero-sub mt-6 max-w-xl font-roboto text-[17px] leading-relaxed text-text-secondary">
            Lifeflow&#8288;-&#8288;chain records every step of a blood unit&rsquo;s
            journey &mdash; donation, screening, storage, delivery &mdash; onto an
            immutable ledger. A blood bank cannot quietly edit a safety record after
            the fact. That is the whole point.
          </p>

          <div className="mt-9 flex flex-col gap-4 sm:flex-row">
            <Link
              href="/track"
              className="hero-cta inline-flex h-[52px] items-center justify-center gap-2 rounded-xl bg-oxblood-800 px-7 font-roboto text-[15px] font-semibold text-white shadow-card-lg transition hover:bg-oxblood-900"
            >
              Track a SmartBag
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" className="stroke-current">
                <path d="M5 12h14M13 6l6 6-6 6" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
              </svg>
            </Link>
            <Link
              href="/#how"
              className="hero-cta inline-flex h-[52px] items-center justify-center rounded-xl border border-oxblood-300 bg-surface px-7 font-roboto text-[15px] font-semibold text-oxblood-800 transition hover:bg-oxblood-50 dark:border-oxblood-800 dark:text-oxblood-300 dark:hover:bg-oxblood-950"
            >
              See how it works
            </Link>
          </div>

          <dl className="mt-12 flex flex-wrap gap-x-10 gap-y-4">
            {[
              ["57%", "fewer transfusion infections"],
              ["3", "countries deployed"],
              ["USSD", "works on feature phones"],
            ].map(([k, v]) => (
              <div key={v} className="hero-stat">
                <dt className="font-manrope text-2xl font-bold text-oxblood-800 dark:text-oxblood-300">
                  {k}
                </dt>
                <dd className="max-w-[9rem] font-poppins text-[12.5px] leading-tight text-text-muted">
                  {v}
                </dd>
              </div>
            ))}
          </dl>
        </div>

        {/* Right: chain-of-custody card */}
        <div className="hero-card relative">
          <div className="hero-drop pointer-events-none absolute -right-6 -top-10 h-24 w-24 rounded-full bg-blood-gradient opacity-90 shadow-blood-drop" />
          <div className="relative rounded-2xl border border-border-muted bg-surface/90 p-6 shadow-card-lg backdrop-blur">
            <div className="flex items-center justify-between border-b border-border-muted pb-4">
              <div>
                <p className="font-poppins text-[11px] uppercase tracking-wider text-text-muted">
                  SmartBag
                </p>
                <p className="font-manrope text-lg font-bold text-text-primary">
                  #SB-2K7F-9410
                </p>
              </div>
              <span className="inline-flex items-center gap-1.5 rounded-full bg-oxblood-50 px-3 py-1 font-poppins text-[12px] font-medium text-oxblood-700 dark:bg-oxblood-950 dark:text-oxblood-300">
                <span className="h-1.5 w-1.5 rounded-full bg-status-resolved" />
                Verified safe
              </span>
            </div>

            <ul className="mt-5 space-y-4">
              {CUSTODY.map((c, i) => (
                <li key={c.step} className="hero-row flex gap-4">
                  <div className="flex flex-col items-center">
                    <span className="flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-oxblood-800 font-manrope text-[12px] font-bold text-white">
                      {i + 1}
                    </span>
                    {i < CUSTODY.length - 1 && (
                      <span className="mt-1 w-px flex-1 bg-gradient-to-b from-oxblood-400 to-transparent" />
                    )}
                  </div>
                  <div className="pb-1">
                    <p className="font-roboto text-[14px] font-semibold text-text-primary">
                      {c.step}
                    </p>
                    <p className="font-poppins text-[12px] leading-snug text-text-muted">
                      {c.who}
                    </p>
                    <p className="mt-0.5 font-mono text-[11px] text-oxblood-600 dark:text-oxblood-400">
                      {c.hash}
                    </p>
                  </div>
                </li>
              ))}
            </ul>

            <p className="mt-5 border-t border-border-muted pt-4 font-poppins text-[11.5px] leading-snug text-text-muted">
              Each event is hashed and anchored on Soroban. Tamper with one record
              and the chain breaks &mdash; visibly.
            </p>
          </div>
        </div>
      </div>
    </section>
  );
}
