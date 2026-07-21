export type ThemeMode = "light" | "dark";

type ThemeTokens = {
  colors: Record<string, string>;
  shadows: Record<string, string>;
};

export const LIGHT_THEME: ThemeTokens = {
  colors: {
    canvas: "250 250 249",
    panel: "255 255 255",
    ink: "15 23 42",
    brand: "13 147 115",

    "sidebar-bg": "249 248 246",
    "sidebar-border": "229 231 235",
    "sidebar-text": "107 114 128",
    "sidebar-text-active": "15 23 42",
    "sidebar-active-bg": "220 252 231",
    "sidebar-hover-bg": "243 244 246",
    "sidebar-title": "15 23 42",

    "bar-bg": "30 41 59",
    "bar-border": "15 23 42",
    "bar-text": "203 213 225",
    "bar-accent": "110 231 183",
    "bar-strong": "255 255 255",

    "text-secondary": "71 85 105",
    "text-muted": "148 163 184",

    border: "229 231 235",
    "border-light": "243 244 246",
  },
  shadows: {
    card: "0 1px 3px 0 rgb(0 0 0 / 0.04), 0 1px 2px -1px rgb(0 0 0 / 0.04)",
    panel: "0 4px 6px -1px rgb(0 0 0 / 0.05), 0 2px 4px -2px rgb(0 0 0 / 0.03)",
    float: "0 10px 25px -5px rgb(0 0 0 / 0.08), 0 8px 10px -6px rgb(0 0 0 / 0.04)",
  },
};

export const DARK_THEME: ThemeTokens = {
  colors: {
    canvas: "15 23 42",
    panel: "30 41 59",
    ink: "248 250 252",
    brand: "52 211 153",

    "sidebar-bg": "15 23 42",
    "sidebar-border": "30 41 59",
    "sidebar-text": "148 163 184",
    "sidebar-text-active": "248 250 252",
    "sidebar-active-bg": "30 41 59",
    "sidebar-hover-bg": "22 33 54",
    "sidebar-title": "248 250 252",

    "bar-bg": "2 6 23",
    "bar-border": "15 23 42",
    "bar-text": "148 163 184",
    "bar-accent": "52 211 153",
    "bar-strong": "226 232 240",

    "text-secondary": "203 213 225",
    "text-muted": "148 163 184",

    border: "51 65 85",
    "border-light": "30 41 59",
  },
  shadows: {
    card: "0 1px 3px 0 rgb(0 0 0 / 0.3), 0 1px 2px -1px rgb(0 0 0 / 0.25)",
    panel: "0 4px 6px -1px rgb(0 0 0 / 0.4), 0 2px 4px -2px rgb(0 0 0 / 0.3)",
    float: "0 10px 25px -5px rgb(0 0 0 / 0.5), 0 8px 10px -6px rgb(0 0 0 / 0.4)",
  },
};

export function applyTheme(mode: ThemeMode) {
  const tokens = mode === "dark" ? DARK_THEME : LIGHT_THEME;
  const root = document.documentElement;

  root.dataset.theme = mode;

  for (const [key, value] of Object.entries(tokens.colors)) {
    root.style.setProperty(`--color-${key}`, value);
  }

  for (const [key, value] of Object.entries(tokens.shadows)) {
    root.style.setProperty(`--shadow-${key}`, value);
  }
}

export function applyStoredTheme() {
  try {
    const saved = window.localStorage.getItem("msh.app.settings");
    if (!saved) {
      applyTheme("light");
      return;
    }
    const parsed = JSON.parse(saved) as { theme?: string };
    applyTheme(parsed.theme === "dark" ? "dark" : "light");
  } catch {
    applyTheme("light");
  }
}
