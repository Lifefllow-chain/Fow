import type { Metadata } from "next";
import Navbar from "../../components/Navbar";
import Footer from "../../components/Footer";
import SmartBagTracker from "../../components/track/SmartBagTracker";

export const metadata: Metadata = {
  title: "Track a SmartBag — Lifeflow-chain Protocol",
  description:
    "Look up the full, tamper-proof chain of custody for a unit of blood: donation, screening, storage, delivery and transfusion.",
};

export default function TrackPage() {
  return (
    <main className="min-h-screen bg-surface">
      <Navbar />

      <section className="relative overflow-hidden pt-32 pb-14 md:pt-40">
        <div className="pointer-events-none absolute inset-0 bg-oxblood-radial" />
        <div className="relative mx-auto max-w-3xl px-6 text-center">
          <span className="inline-flex items-center gap-2 rounded-full border border-oxblood-200 bg-oxblood-50 px-4 py-1.5 font-poppins text-[13px] font-medium text-oxblood-700 dark:border-oxblood-800 dark:bg-oxblood-950 dark:text-oxblood-300">
            Vein-to-vein lookup
          </span>
          <h1 className="mt-5 font-manrope text-[36px] font-bold leading-tight text-text-primary md:text-[48px]">
            Trust what you&rsquo;re transfusing.
          </h1>
          <p className="mx-auto mt-4 max-w-xl font-roboto text-[16px] leading-relaxed text-text-secondary">
            Scan or enter a SmartBag ID to see every step of that unit&rsquo;s
            journey &mdash; recorded once, on-chain, and impossible to quietly
            rewrite.
          </p>
        </div>
      </section>

      <section className="pb-24">
        <SmartBagTracker />
      </section>

      <Footer />
    </main>
  );
}
