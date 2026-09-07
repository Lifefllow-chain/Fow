import Image from "next/image";
import Link from "next/link";

const COLS = [
  {
    title: "Protocol",
    links: [
      ["On-chain escrow", "/#solution"],
      ["Supply-chain events", "/#how"],
      ["Actor registry", "/#solution"],
      ["Transparency dashboard", "/transparency"],
    ],
  },
  {
    title: "Build",
    links: [
      ["Track a SmartBag", "/track"],
      ["Enter the app", "/auth/signin"],
      ["Register your org", "/auth/signup"],
      ["Docs & contracts", "/#developers"],
    ],
  },
  {
    title: "About",
    links: [
      ["The blood-safety problem", "/#problem"],
      ["Impact & evaluation", "/#impact"],
      ["Low-tech access (USSD)", "/#access"],
      ["Open source", "/#developers"],
    ],
  },
];

export default function Footer() {
  return (
    <footer className="w-full bg-oxblood-950 text-oxblood-100 font-dmsans">
      <div className="mx-auto max-w-[1288px] px-6 pt-16 pb-10">
        {/* CTA row */}
        <div className="flex flex-col items-center justify-between gap-6 border-b border-oxblood-800/60 pb-12 md:flex-row">
          <div className="flex items-center gap-4">
            <span className="flex h-14 w-14 items-center justify-center rounded-full bg-white/95">
              <Image src="/logo-drop.svg" alt="" width={28} height={28} />
            </span>
            <div>
              <p className="font-manrope text-lg font-bold text-white">
                Lifeflow&#8288;-&#8288;chain Protocol
              </p>
              <p className="text-[13px] text-oxblood-200/80">
                Tamper-proof chain-of-custody for blood.
              </p>
            </div>
          </div>
          <div className="flex flex-col items-center gap-4 sm:flex-row">
            <span className="text-[15px] text-oxblood-100/90">
              Building blood infrastructure?
            </span>
            <Link href="/auth/signup">
              <button className="rounded-lg bg-white px-6 py-3 font-roboto text-[15px] font-bold text-oxblood-900 shadow-lg transition hover:bg-oxblood-50">
                Register your organisation
              </button>
            </Link>
          </div>
        </div>

        {/* Link columns */}
        <div className="grid grid-cols-1 gap-12 py-14 sm:grid-cols-2 lg:grid-cols-[1.2fr_repeat(3,1fr)]">
          <div className="flex flex-col gap-5">
            <p className="max-w-[15rem] text-[15px] text-oxblood-100/90">
              Get protocol updates and deployment notes.
            </p>
            <form className="flex h-[50px] w-full max-w-[300px] items-center overflow-hidden rounded-lg border border-oxblood-700 bg-oxblood-900">
              <input
                type="email"
                placeholder="Email address"
                aria-label="Email address"
                className="h-full w-full border-none bg-transparent px-4 font-poppins text-[14px] text-white placeholder-oxblood-300 focus:outline-none"
              />
              <button
                type="submit"
                aria-label="Subscribe"
                className="flex h-full w-12 items-center justify-center bg-oxblood-700 transition hover:bg-oxblood-600"
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" className="stroke-white">
                  <path d="M5 12h14M13 6l6 6-6 6" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
                </svg>
              </button>
            </form>
          </div>

          {COLS.map((col) => (
            <div key={col.title}>
              <h3 className="mb-5 font-poppins text-[13px] uppercase tracking-wider text-oxblood-300">
                {col.title}
              </h3>
              <ul className="space-y-3 text-[14px] text-oxblood-100/85">
                {col.links.map(([label, href]) => (
                  <li key={label}>
                    <Link href={href} className="transition hover:text-white">
                      {label}
                    </Link>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>

        {/* bottom row */}
        <div className="flex flex-col items-center justify-between gap-5 border-t border-oxblood-800/60 pt-8 md:flex-row">
          <p className="text-[13px] text-oxblood-200/70">
            © {new Date().getFullYear()} Lifeflow-chain Protocol · Open source, community-governed.
          </p>
          <div className="flex gap-6 text-[13px] text-oxblood-200/80">
            <Link href="/#" className="transition hover:text-white">Terms</Link>
            <Link href="/#" className="transition hover:text-white">Privacy</Link>
            <Link href="/transparency" className="transition hover:text-white">Audit log</Link>
          </div>
        </div>
      </div>
    </footer>
  );
}
