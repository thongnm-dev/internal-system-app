/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        ink: "#1e252b",
        panel: "#ffffff",
        canvas: "#f4f1ea",
        brand: "#1f6f63",
      },
    },
  },
  plugins: [],
};
