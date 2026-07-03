/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{vue,ts,js}"],
  theme: {
    extend: {
      colors: {
        ink: "rgb(var(--color-ink) / <alpha-value>)",
        panel: "rgb(var(--color-panel) / <alpha-value>)",
        canvas: "rgb(var(--color-canvas) / <alpha-value>)",
        brand: "rgb(var(--color-brand) / <alpha-value>)",

        sidebar: {
          DEFAULT: "rgb(var(--color-sidebar-bg) / <alpha-value>)",
          border: "rgb(var(--color-sidebar-border) / <alpha-value>)",
          text: "rgb(var(--color-sidebar-text) / <alpha-value>)",
          "text-active": "rgb(var(--color-sidebar-text-active) / <alpha-value>)",
          active: "rgb(var(--color-sidebar-active-bg) / <alpha-value>)",
          hover: "rgb(var(--color-sidebar-hover-bg) / <alpha-value>)",
          title: "rgb(var(--color-sidebar-title) / <alpha-value>)",
        },

        bar: {
          DEFAULT: "rgb(var(--color-bar-bg) / <alpha-value>)",
          border: "rgb(var(--color-bar-border) / <alpha-value>)",
          text: "rgb(var(--color-bar-text) / <alpha-value>)",
          accent: "rgb(var(--color-bar-accent) / <alpha-value>)",
          strong: "rgb(var(--color-bar-strong) / <alpha-value>)",
        },

        secondary: "rgb(var(--color-text-secondary) / <alpha-value>)",
        muted: "rgb(var(--color-text-muted) / <alpha-value>)",
        divider: "rgb(var(--color-border) / <alpha-value>)",
        "divider-light": "rgb(var(--color-border-light) / <alpha-value>)",
      },
      boxShadow: {
        card: "var(--shadow-card)",
        "card-panel": "var(--shadow-panel)",
        float: "var(--shadow-float)",
      },
    },
  },
  plugins: [],
};
