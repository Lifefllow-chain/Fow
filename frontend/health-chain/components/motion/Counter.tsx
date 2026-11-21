"use client";

import { useRef } from "react";
import gsap from "gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";
import { useGSAP } from "@gsap/react";

gsap.registerPlugin(ScrollTrigger);

type Props = {
  to: number;
  prefix?: string;
  suffix?: string;
  decimals?: number;
  className?: string;
};

/** Counts up from 0 → `to` when scrolled into view. Static under reduced motion. */
export default function Counter({
  to,
  prefix = "",
  suffix = "",
  decimals = 0,
  className,
}: Props) {
  const ref = useRef<HTMLSpanElement>(null);

  useGSAP(
    () => {
      const el = ref.current;
      if (!el) return;
      const fmt = (v: number) =>
        `${prefix}${v.toLocaleString(undefined, {
          minimumFractionDigits: decimals,
          maximumFractionDigits: decimals,
        })}${suffix}`;

      if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
        el.textContent = fmt(to);
        return;
      }

      const obj = { v: 0 };
      el.textContent = fmt(0);
      gsap.to(obj, {
        v: to,
        duration: 2,
        ease: "power2.out",
        onUpdate: () => {
          el.textContent = fmt(obj.v);
        },
        scrollTrigger: { trigger: el, start: "top 88%", once: true },
      });
    },
    { scope: ref, dependencies: [to] }
  );

  return (
    <span ref={ref} className={className}>
      {prefix}
      {to}
      {suffix}
    </span>
  );
}
