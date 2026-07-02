import type { App } from "vue";
import { createPinia } from "pinia";
import PrimeVue from "primevue/config";
import Aura from "@primevue/themes/aura";

export function registerPlugins(app: App) {
  app.use(createPinia());
  app.use(PrimeVue, {
    theme: {
      preset: Aura,
      options: {
        darkModeSelector: "[data-theme='dark']",
      },
    },
    ripple: true,
  });
}
