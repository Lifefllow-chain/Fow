import type { Metadata } from "next";
import React, { Suspense } from "react";
import { Poppins, Roboto, Manrope, DM_Sans } from "next/font/google";
import "./globals.css";
import { ToastProvider } from "../components/providers/ToastProvider";
import { ReactQueryProvider } from "../components/providers/ReactQueryProvider";
import { I18nProvider } from "../components/providers/I18nProvider";
import { WalletProvider } from "../components/providers/WalletProvider";
import MotionProvider from "../components/motion/MotionProvider";
import { ThemeProvider } from "../components/providers/ThemeProvider";
import { Toaster } from "sonner";
import NetworkMismatchBanner from "../components/blockchain/NetworkMismatchBanner";
import { SkipLink } from "../components/accessibility/AccessibleComponents";
import { OfflineBanner } from "../components/ui/OfflineBanner";

const poppins = Poppins({
  subsets: ["latin"],
  weight: ["400"],
  variable: "--font-poppins",
});

const roboto = Roboto({
  subsets: ["latin"],
  weight: ["400", "600", "700"],
  variable: "--font-roboto",
});

const manrope = Manrope({
  subsets: ["latin"],
  weight: ["600", "700"],
  variable: "--font-manrope",
});

const dmSans = DM_Sans({
  subsets: ["latin"],
  weight: ["400", "700"],
  variable: "--font-dm-sans",
});

export const metadata: Metadata = {
  title: "Lifeflow-chain Protocol — Vein-to-vein blood traceability",
  description:
    "An open-source protocol on Stellar Soroban for tamper-proof blood chain-of-custody, transparent health donations, and immutable healthcare supply-chain tracking.",
  manifest: "/manifest.json",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <link rel="manifest" href="/manifest.json" />
        <meta name="theme-color" content="#420e10" />
        {/* Apply the saved theme before first paint to avoid a flash. */}
        <script
          dangerouslySetInnerHTML={{
            __html: `(function(){try{var t=localStorage.getItem('theme')||'system';var d=t==='dark'||(t!=='light'&&window.matchMedia('(prefers-color-scheme: dark)').matches);document.documentElement.classList.toggle('dark',d);document.documentElement.style.colorScheme=d?'dark':'light';}catch(e){}})();`,
          }}
        />
      </head>
      <body
        className={`${poppins.variable} ${roboto.variable} ${manrope.variable} ${dmSans.variable} antialiased bg-surface text-text-primary`}
      >
        <SkipLink href="#main-content" />
        <Suspense fallback={null}>
          <ThemeProvider>
            <I18nProvider>
              <ReactQueryProvider>
                <WalletProvider>
                  <OfflineBanner />
                  <ToastProvider>
                    <MotionProvider>{children}</MotionProvider>
                  </ToastProvider>
                  <NetworkMismatchBanner />
                  <Toaster position="top-right" richColors theme="system" />
                </WalletProvider>
              </ReactQueryProvider>
            </I18nProvider>
          </ThemeProvider>
        </Suspense>
      </body>
    </html>
  );
}
