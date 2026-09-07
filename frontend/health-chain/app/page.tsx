import Link from "next/link";
import {
  ShieldCheck,
  Lock,
  Activity,
  Boxes,
  UserCheck,
  Timer,
  EyeOff,
  QrCode,
  Phone,
  GitBranch,
  Microscope,
  Truck,
} from "lucide-react";
import Navbar from "../components/Navbar";
import Hero from "../components/Hero";
import Footer from "../components/Footer";
import Counter from "../components/motion/Counter";

const PROBLEMS = [
  "Donors cannot verify where funds go or whether supplies reach recipients.",
  "In Nigeria, a large share of new HIV infections trace back to unsafe transfusions.",
  "Safety records can be edited after the fact — so there is little pressure to test rigorously.",
  "Supply chains for blood, vaccines and medical goods are hard to audit end-to-end.",
];

const FEATURES = [
  { icon: Lock, title: "On-chain escrow", body: "Medical donations are held by a Soroban contract and released only when real-world conditions are met." },
  { icon: Activity, title: "Donor impact tracking", body: "Every donor can follow their contribution to the unit, shipment or patient it supported." },
  { icon: Boxes, title: "Immutable supply-chain events", body: "Collection, screening, storage and delivery are written once and never rewritten." },
  { icon: UserCheck, title: "Verified actor registry", body: "Hospitals, blood banks, labs and NGOs are registered and attested before they can log events." },
  { icon: Timer, title: "Time-locked releases", body: "Funds unlock on a schedule or on verified milestones — not on a single administrator's say-so." },
  { icon: EyeOff, title: "Privacy-preserving IDs", body: "Donors and patients are referenced by rotating codes, never by identifying data on-chain." },
];

const STEPS = [
  { icon: ShieldCheck, title: "Donation & collection", body: "A donor gives blood into a SmartBag. Donor code, site and collector are logged." },
  { icon: Microscope, title: "Screening", body: "HIV, HBV, HCV and syphilis tests are recorded with the technician and result — permanently." },
  { icon: Boxes, title: "Storage", body: "Cold-chain temperature and location are anchored while the unit sits in the bank." },
  { icon: Truck, title: "Delivery", body: "The sealed, temperature-logged unit moves to the requesting hospital with a signed handover." },
  { icon: QrCode, title: "Bedside verification", body: "The ward scans the QR code and sees the unit's full, unalterable history before transfusing." },
];

export default function Home() {
  return (
    <main className="min-h-screen overflow-x-hidden bg-surface">
      <Navbar />
      <Hero />

      {/* trust strip */}
      <section className="border-y border-border-muted bg-surface-raised/60">
        <div
          className="mx-auto flex max-w-[1288px] flex-wrap items-center justify-center gap-x-10 gap-y-3 px-6 py-5 font-poppins text-[13px] text-text-muted"
          data-animate="fade"
        >
          <span>Stellar network</span>
          <span className="h-1 w-1 rounded-full bg-oxblood-300" />
          <span>Soroban smart contracts</span>
          <span className="h-1 w-1 rounded-full bg-oxblood-300" />
          <span>Independent evaluation by NIMR</span>
          <span className="h-1 w-1 rounded-full bg-oxblood-300" />
          <span>Deployed in Nigeria · Kenya · Sierra Leone</span>
        </div>
      </section>

      {/* problem */}
      <section id="problem" className="mx-auto max-w-[1288px] px-6 py-24 md:py-32">
        <div className="grid gap-14 lg:grid-cols-[0.9fr_1.1fr]">
          <div data-animate="left">
            <p className="font-poppins text-[13px] font-semibold uppercase tracking-wider text-oxblood-700 dark:text-oxblood-400">
              The problem
            </p>
            <h2 className="mt-3 font-manrope text-[34px] font-bold leading-tight text-text-primary md:text-[42px]">
              The core problem is blood safety &mdash; not payments, not crypto.
            </h2>
            <p className="mt-5 font-roboto text-[16px] leading-relaxed text-text-secondary">
              Healthcare donation and supply systems suffer from a lack of
              transparency, centralised control, and poor auditability. When a blood
              bank can quietly rewrite a safety record, nothing forces rigorous
              testing.
            </p>
          </div>
          <ul className="grid gap-4 sm:grid-cols-2" data-animate-stagger="0.12" data-animate>
            {PROBLEMS.map((p) => (
              <li
                key={p}
                className="rounded-xl border border-border-muted bg-surface p-5 font-roboto text-[14.5px] leading-relaxed text-text-secondary shadow-card"
              >
                <span className="mb-3 block font-manrope text-lg text-oxblood-600 dark:text-oxblood-400">&#10007;</span>
                {p}
              </li>
            ))}
          </ul>
        </div>
      </section>

      {/* solution / features */}
      <section id="solution" className="bg-oxblood-950 py-24 text-oxblood-50 md:py-32">
        <div className="mx-auto max-w-[1288px] px-6">
          <div className="max-w-2xl" data-animate="up">
            <p className="font-poppins text-[13px] font-semibold uppercase tracking-wider text-oxblood-300">
              The protocol
            </p>
            <h2 className="mt-3 font-manrope text-[34px] font-bold leading-tight text-white md:text-[42px]">
              Critical actions are enforced by smart contracts, not people.
            </h2>
            <p className="mt-5 font-roboto text-[16px] leading-relaxed text-oxblood-100/85">
              Lifeflow-chain uses Stellar and Soroban to make each donation
              traceable, auditable, and releasable only when real-world conditions
              are met.
            </p>
          </div>

          <div
            className="mt-14 grid gap-5 sm:grid-cols-2 lg:grid-cols-3"
            data-animate
            data-animate-stagger="0.1"
          >
            {FEATURES.map((f) => (
              <div
                key={f.title}
                className="rounded-2xl border border-oxblood-800/70 bg-oxblood-900/50 p-6 transition hover:border-oxblood-600 hover:bg-oxblood-900"
              >
                <span className="flex h-11 w-11 items-center justify-center rounded-xl bg-oxblood-800 text-oxblood-100">
                  <f.icon size={20} strokeWidth={1.75} />
                </span>
                <h3 className="mt-4 font-manrope text-[17px] font-bold text-white">
                  {f.title}
                </h3>
                <p className="mt-2 font-poppins text-[13.5px] leading-relaxed text-oxblood-100/75">
                  {f.body}
                </p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* how it works — vein to vein */}
      <section id="how" className="mx-auto max-w-[1288px] px-6 py-24 md:py-32">
        <div className="max-w-2xl" data-animate="up">
          <p className="font-poppins text-[13px] font-semibold uppercase tracking-wider text-oxblood-700 dark:text-oxblood-400">
            How SmartBag works
          </p>
          <h2 className="mt-3 font-manrope text-[34px] font-bold leading-tight text-text-primary md:text-[42px]">
            &ldquo;Vein to vein&rdquo; traceability for a single unit of blood.
          </h2>
          <p className="mt-5 font-roboto text-[16px] leading-relaxed text-text-secondary">
            When a hospital scans a SmartBag, they see the full history of that unit
            &mdash; donation, collection, screening, storage and delivery. Because it
            lives on a distributed ledger, the record cannot be quietly altered.
          </p>
        </div>

        <ol className="mt-16 grid gap-6 md:grid-cols-5" data-animate data-animate-stagger="0.12">
          {STEPS.map((s, i) => (
            <li key={s.title} className="relative">
              <div className="flex items-center gap-3">
                <span className="flex h-10 w-10 items-center justify-center rounded-full bg-oxblood-800 font-manrope text-sm font-bold text-white">
                  {i + 1}
                </span>
                {i < STEPS.length - 1 && (
                  <span className="hidden h-px flex-1 bg-gradient-to-r from-oxblood-400 to-transparent md:block" />
                )}
              </div>
              <span className="mt-4 flex h-9 w-9 items-center justify-center rounded-lg bg-oxblood-50 text-oxblood-700 dark:bg-oxblood-950 dark:text-oxblood-300">
                <s.icon size={18} strokeWidth={1.75} />
              </span>
              <h3 className="mt-3 font-manrope text-[16px] font-bold text-text-primary">
                {s.title}
              </h3>
              <p className="mt-1.5 font-poppins text-[13px] leading-relaxed text-text-muted">
                {s.body}
              </p>
            </li>
          ))}
        </ol>

        <div
          className="mt-16 flex flex-col items-start gap-4 rounded-2xl border border-oxblood-200 bg-oxblood-50 p-7 dark:border-oxblood-800 dark:bg-oxblood-950 md:flex-row md:items-center md:justify-between"
          data-animate="scale"
        >
          <p className="max-w-2xl font-roboto text-[15px] leading-relaxed text-oxblood-900 dark:text-oxblood-100">
            Because every step is on-chain, a blood bank cannot fake or edit a
            safety record after the fact &mdash; which pushes them toward more
            rigorous testing, because everything is on the record.
          </p>
          <Link
            href="/track"
            className="inline-flex h-11 shrink-0 items-center justify-center gap-2 rounded-xl bg-oxblood-800 px-6 font-roboto text-sm font-semibold text-white transition hover:bg-oxblood-900"
          >
            <QrCode size={16} /> Try the tracker
          </Link>
        </div>
      </section>

      {/* low-tech access */}
      <section id="access" className="bg-surface-raised/60 py-24 md:py-28">
        <div className="mx-auto grid max-w-[1288px] items-center gap-14 px-6 lg:grid-cols-2">
          <div data-animate="left">
            <p className="font-poppins text-[13px] font-semibold uppercase tracking-wider text-oxblood-700 dark:text-oxblood-400">
              Built for the Nigerian landscape
            </p>
            <h2 className="mt-3 font-manrope text-[32px] font-bold leading-tight text-text-primary md:text-[40px]">
              High-tech ledger, low-tech access.
            </h2>
            <p className="mt-5 font-roboto text-[16px] leading-relaxed text-text-secondary">
              The protocol does not assume everyone has a smartphone or internet. A
              rural clinic can pull up a unit&rsquo;s history through USSD short
              codes on a feature phone &mdash; the same immutable record, no app
              required.
            </p>
          </div>
          <div
            className="rounded-2xl border border-border-muted bg-surface p-6 shadow-card"
            data-animate="right"
          >
            <div className="flex items-center gap-3 border-b border-border-muted pb-4">
              <Phone size={18} className="text-oxblood-700 dark:text-oxblood-300" />
              <span className="font-manrope text-sm font-bold text-text-primary">
                USSD session
              </span>
            </div>
            <pre className="mt-4 whitespace-pre-wrap font-mono text-[13px] leading-relaxed text-text-secondary">
{`*347*5#
LIFEFLOW — Verify a unit
Enter SmartBag ID: SB2K7F9410

Unit SB-2K7F-9410
✔ Screened: HIV/HBV/HCV/Syph — negative
✔ Cold chain: OK (2–6°C)
✔ Delivered sealed — 12 Aug, 14:20
Status: SAFE TO TRANSFUSE`}
            </pre>
          </div>
        </div>
      </section>

      {/* impact */}
      <section id="impact" className="mx-auto max-w-[1288px] px-6 py-24 md:py-32">
        <div className="max-w-2xl" data-animate="up">
          <p className="font-poppins text-[13px] font-semibold uppercase tracking-wider text-oxblood-700 dark:text-oxblood-400">
            Impact
          </p>
          <h2 className="mt-3 font-manrope text-[34px] font-bold leading-tight text-text-primary md:text-[42px]">
            Not just marketing &mdash; independently evaluated.
          </h2>
        </div>

        <div className="mt-14 grid gap-6 sm:grid-cols-3" data-animate data-animate-stagger="0.12">
          {[
            { v: <Counter to={57} suffix="%" />, label: "reduction in transfusion-transmissible infections vs. standard practice, per Nigeria's Institute of Medical Research." },
            { v: <Counter to={3} />, label: "countries with active deployments — Nigeria, Kenya and Sierra Leone — via the SmartBank platform." },
            { v: <span className="text-gradient-oxblood">∞</span>, label: "the ledger is append-only: safety records cannot be rewritten once anchored on Soroban." },
          ].map((s, i) => (
            <div
              key={i}
              className="rounded-2xl border border-border-muted bg-surface p-7 shadow-card"
            >
              <p className="font-manrope text-[44px] font-bold leading-none text-oxblood-800 dark:text-oxblood-300">
                {s.v}
              </p>
              <p className="mt-4 font-poppins text-[13.5px] leading-relaxed text-text-muted">
                {s.label}
              </p>
            </div>
          ))}
        </div>
      </section>

      {/* developers / open source */}
      <section id="developers" className="border-t border-border-muted bg-oxblood-950 py-20 text-oxblood-50">
        <div
          className="mx-auto flex max-w-[1288px] flex-col items-center gap-8 px-6 text-center"
          data-animate="up"
        >
          <span className="flex h-12 w-12 items-center justify-center rounded-xl bg-oxblood-800">
            <GitBranch size={22} />
          </span>
          <h2 className="max-w-2xl font-manrope text-[30px] font-bold leading-tight text-white md:text-[38px]">
            Open source, and built to be forked.
          </h2>
          <p className="max-w-xl font-roboto text-[15.5px] leading-relaxed text-oxblood-100/85">
            The Soroban contracts, the traceability layer and this interface are all
            open. Register your hospital or blood bank, or explore the public
            transparency dashboard.
          </p>
          <div className="flex flex-col gap-4 sm:flex-row">
            <Link
              href="/auth/signup"
              className="inline-flex h-12 items-center justify-center rounded-xl bg-white px-7 font-roboto text-sm font-semibold text-oxblood-900 transition hover:bg-oxblood-50"
            >
              Register your organisation
            </Link>
            <Link
              href="/transparency"
              className="inline-flex h-12 items-center justify-center rounded-xl border border-oxblood-700 px-7 font-roboto text-sm font-semibold text-white transition hover:bg-oxblood-900"
            >
              Open transparency dashboard
            </Link>
          </div>
        </div>
      </section>

      <Footer />
    </main>
  );
}
