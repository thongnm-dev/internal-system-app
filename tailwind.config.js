/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{vue,ts,js}"],
  darkMode: ["selector", "[data-theme='dark']"],
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

        danger: "rgb(var(--color-danger) / <alpha-value>)",
        "danger-soft": "rgb(var(--color-danger-soft) / <alpha-value>)",
        "danger-border": "rgb(var(--color-danger-border) / <alpha-value>)",
        warning: "rgb(var(--color-warning) / <alpha-value>)",
        "warning-soft": "rgb(var(--color-warning-soft) / <alpha-value>)",
        "warning-border": "rgb(var(--color-warning-border) / <alpha-value>)",
        success: "rgb(var(--color-success) / <alpha-value>)",
        "success-soft": "rgb(var(--color-success-soft) / <alpha-value>)",
        "success-border": "rgb(var(--color-success-border) / <alpha-value>)",
        info: "rgb(var(--color-info) / <alpha-value>)",
        "info-soft": "rgb(var(--color-info-soft) / <alpha-value>)",
        "info-border": "rgb(var(--color-info-border) / <alpha-value>)",

        "on-brand": "rgb(var(--color-on-brand) / <alpha-value>)",
        code: "rgb(var(--color-code-bg) / <alpha-value>)",
        "code-fg": "rgb(var(--color-code-fg) / <alpha-value>)",
      },
      fontSize: {
        "2xs": ["0.6875rem", { lineHeight: "1rem" }], // 11px
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
