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
      },
    },
  },
  plugins: [],
};
