"use client";

/**
 * Global GSAP wiring for the marketing surface.
 *
 * Reveals any element tagged `data-animate` as it scrolls into view.
 * `data-animate="left|right|up|scale|fade"` picks the motion (default "up");
 * `data-animate-delay` (s) and `data-animate-stagger` (container whose direct
 * children animate in sequence) fine-tune it. Respects `prefers-reduced-motion`.
 */

import gsap from "gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";
import { useGSAP } from "@gsap/react";

gsap.registerPlugin(ScrollTrigger);

const FROM: Record<string, gsap.TweenVars> = {
  up: { y: 48, opacity: 0 },
  fade: { opacity: 0 },
  left: { x: -56, opacity: 0 },
  right: { x: 56, opacity: 0 },
  scale: { scale: 0.92, opacity: 0 },
};

export default function MotionProvider({
  children,
}: {
  children: React.ReactNode;
}) {
  useGSAP(() => {
    document.documentElement.classList.add("gsap-ready");

    const els = gsap.utils.toArray<HTMLElement>("[data-animate]");
    const reduced = window.matchMedia(
      "(prefers-reduced-motion: reduce)"
    ).matches;

    if (reduced) {
      gsap.set(els, { clearProps: "all" });
      return;
    }

    els.forEach((el) => {
      const kind = el.dataset.animate || "up";
      const delay = parseFloat(el.dataset.animateDelay || "0");
      const stagger = el.dataset.animateStagger;
      const targets = stagger ? (Array.from(el.children) as HTMLElement[]) : el;

      gsap.set(targets, { ...(FROM[kind] ?? FROM.up) });

      const reveal = (instant = false) =>
        gsap.to(targets, {
          x: 0,
          y: 0,
          scale: 1,
          opacity: 1,
          duration: instant ? 0 : 0.9,
          delay: instant ? 0 : delay,
          ease: "power3.out",
          stagger: instant ? 0 : stagger ? parseFloat(stagger) || 0.12 : 0,
          overwrite: true,
        });

      const st = ScrollTrigger.create({
        trigger: el,
        start: "top 85%",
        onEnter: () => reveal(),
      });

      // Already on-screen when the page loaded — reveal without waiting for scroll.
      if (st.progress > 0 || el.getBoundingClientRect().top < window.innerHeight * 0.9) {
        reveal();
      }
    });

    // Single recompute once fonts have loaded and shifted the layout.
    const refresh = () => ScrollTrigger.refresh();
    if (document.fonts?.ready) document.fonts.ready.then(refresh);
  }, []);

  return <>{children}</>;
}
