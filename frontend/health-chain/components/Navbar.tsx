"use client";
import { useEffect, useState } from "react";
import Image from "next/image";
import Link from "next/link";
import ConnectWalletButton from "./blockchain/ConnectWalletButton";
import { ThemeToggle } from "./providers/ThemeToggle";

const LINKS = [
  { href: "/#problem", label: "Problem" },
  { href: "/#solution", label: "Protocol" },
  { href: "/#how", label: "How it works" },
  { href: "/track", label: "Track a SmartBag" },
  { href: "/transparency", label: "Transparency" },
];

export default function Navbar() {
  const [isOpen, setIsOpen] = useState(false);
  const [scrolled, setScrolled] = useState(false);

  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 12);
    onScroll();
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  return (
    <nav className="fixed top-0 left-0 w-full z-50">
      <div
        className={`transition-all duration-300 ${
          scrolled
            ? "bg-surface/85 backdrop-blur-md border-b border-border-muted shadow-[0_8px_30px_rgba(66,14,16,0.06)]"
            : "bg-transparent border-b border-transparent"
        }`}
      >
        <div className="w-full max-w-[1288px] mx-auto flex items-center justify-between px-6 py-3">
          {/* Logo */}
          <Link href="/" className="relative z-50 flex items-center gap-3 shrink-0">
            <span className="w-11 h-11 rounded-full bg-oxblood-900 flex items-center justify-center shadow-md ring-1 ring-oxblood-700/40">
              <Image src="/logo-drop.svg" alt="" width={22} height={22} className="invert brightness-0" />
            </span>
            <span className="hidden sm:flex flex-col leading-tight">
              <span className="font-manrope font-bold text-[15px] text-text-primary tracking-tight">
                Lifeflow&#8288;-&#8288;chain
              </span>
              <span className="font-poppins text-[11px] text-text-muted -mt-0.5">
                vein-to-vein protocol
              </span>
            </span>
          </Link>

          {/* Hamburger */}
          <button
            className="lg:hidden z-50 text-text-primary p-2"
            onClick={() => setIsOpen(!isOpen)}
            aria-label={isOpen ? "Close menu" : "Open menu"}
            aria-expanded={isOpen}
          >
            {isOpen ? (
              <svg className="w-7 h-7" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            ) : (
              <svg className="w-7 h-7" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
              </svg>
            )}
          </button>

          {/* Desktop nav */}
          <div className="hidden lg:flex items-center gap-8 font-poppins text-[15px] text-text-secondary">
            {LINKS.map((l) => (
              <Link
                key={l.href}
                href={l.href}
                className="group relative py-1.5 hover:text-oxblood-700 transition-colors"
              >
                {l.label}
                <span className="absolute left-0 -bottom-0.5 h-[2px] w-0 bg-oxblood-700 group-hover:w-full transition-all duration-300" />
              </Link>
            ))}
          </div>

          <div className="hidden lg:flex items-center gap-3">
            <ConnectWalletButton />
            <ThemeToggle />
            <Link href="/auth/signin">
              <button className="bg-oxblood-800 hover:bg-oxblood-900 text-white px-5 h-10 rounded-lg shadow-md transition font-roboto font-semibold text-sm">
                Enter App
              </button>
            </Link>
          </div>
        </div>
      </div>

      {/* Mobile dropdown */}
      {isOpen && (
        <div className="lg:hidden bg-surface border-t border-border-muted shadow-xl flex flex-col items-stretch gap-1 px-6 py-6 font-poppins">
          {LINKS.map((l) => (
            <Link
              key={l.href}
              href={l.href}
              onClick={() => setIsOpen(false)}
              className="py-3 text-[17px] text-text-primary border-b border-border-muted/60"
            >
              {l.label}
            </Link>
          ))}
          <div className="flex items-center gap-3 pt-4">
            <ConnectWalletButton />
            <ThemeToggle />
          </div>
          <Link href="/auth/signin" onClick={() => setIsOpen(false)} className="pt-3">
            <button className="w-full bg-oxblood-800 text-white h-11 rounded-lg font-roboto font-semibold">
              Enter App
            </button>
          </Link>
        </div>
      )}
    </nav>
  );
}
