import type { Config } from "tailwindcss";

const config: Config = {
  darkMode: "class",
  content: [
    "./pages/**/*.{js,ts,jsx,tsx,mdx}",
    "./components/**/*.{js,ts,jsx,tsx,mdx}",
    "./app/**/*.{js,ts,jsx,tsx,mdx}",
  ],
  theme: {
    extend: {
      colors: {
        // Semantic tokens — map to CSS variables so both themes work
        surface: "var(--bg-surface)",
        "surface-raised": "var(--bg-surface-raised)",
        "text-primary": "var(--text-primary)",
        "text-secondary": "var(--text-secondary)",
        "text-muted": "var(--text-muted)",
        "border-muted": "var(--border-muted)",
        "status-info": "var(--status-info)",
        "status-warning": "var(--status-warning)",
        "status-critical": "var(--status-critical)",
        "status-resolved": "var(--status-resolved)",

        // ── Oxblood brand scale ──
        oxblood: {
          50: "#fbf1f1",
          100: "#f6dede",
          200: "#ecbcbd",
          300: "#df9091",
          400: "#cf6062",
          500: "#b83f42",
          600: "#9c2b2e",
          700: "#7d1f22",
          800: "#5c1416",
          900: "#420e10",
          950: "#2a0708",
        },

        // Legacy brand aliases retargeted to the oxblood system
        brand: {
          black: "#2a0708",
          dark: "#420e10",
          textBold: "#5c1416",
          navLine: "#7d1f22",
          loginBtn: "#7d1f22",
          requestBtn: "#420e10",
          footer: "#2a0708",
        },
        burgundy: {
          800: "#7d1f22",
          950: "#420e10",
        },
      },
      backgroundImage: {
        "blood-gradient": "linear-gradient(135deg, #7d1f22 0%, #420e10 100%)",
        "oxblood-radial":
          "radial-gradient(1200px 600px at 15% 0%, rgba(125,31,34,0.18), transparent 60%)",
      },
      boxShadow: {
        "blood-drop": "0px 4px 4px 0px rgba(66,14,16,0.35)",
        card: "0px 10px 40px rgba(66,14,16,0.08)",
        "card-lg": "0px 24px 70px rgba(66,14,16,0.14)",
      },
      fontFamily: {
        poppins: ["var(--font-poppins)"],
        roboto: ["var(--font-roboto)"],
        manrope: ["var(--font-manrope)"],
        dmsans: ["var(--font-dm-sans)"],
      },
      keyframes: {
        "pulse-ring": {
          "0%": { transform: "scale(0.9)", opacity: "0.7" },
          "70%": { transform: "scale(1.6)", opacity: "0" },
          "100%": { transform: "scale(1.6)", opacity: "0" },
        },
      },
      animation: {
        "pulse-ring": "pulse-ring 2.4s cubic-bezier(0.215,0.61,0.355,1) infinite",
      },
    },
  },
  plugins: [
    function ({ addUtilities }: any) {
      addUtilities({
        ".sr-only": {
          position: "absolute",
          width: "1px",
          height: "1px",
          padding: "0",
          margin: "-1px",
          overflow: "hidden",
          clip: "rect(0, 0, 0, 0)",
          whiteSpace: "nowrap",
          borderWidth: "0",
        },
        ".focus-visible\\:not-sr-only:focus-visible": {
          position: "static",
          width: "auto",
          height: "auto",
          padding: "inherit",
          margin: "inherit",
          overflow: "visible",
          clip: "auto",
          whiteSpace: "normal",
        },
      });
    },
  ],
};
export default config;
