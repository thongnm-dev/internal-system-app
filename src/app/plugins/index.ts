import type { App } from "vue";
import { createPinia } from "pinia";
import PrimeVue from "primevue/config";
import { definePreset } from "@primevue/themes";
import Aura from "@primevue/themes/aura";
import Tooltip from "primevue/tooltip";

// Thu nhỏ mọi PrimeVue Button trong app (mặc định + size="small") — chỉnh riêng
// token của component `button`, không đụng `formField` nên input/select không đổi.
const CompactAura = definePreset(Aura, {
  components: {
    button: {
      paddingX: "0.4292rem",
      paddingY: "0.1792rem",
      gap: "0.2rem",
      iconOnlyWidth: "1.8667rem",
      sm: {
        paddingX: "0.3667rem",
        paddingY: "0.1167rem",
        iconOnlyWidth: "1.6167rem",
      },
    },
  },
});

export function registerPlugins(app: App) {
  app.use(createPinia());
  app.use(PrimeVue, {
    theme: {
      preset: CompactAura,
      options: {
        darkModeSelector: "[data-theme='dark']",
      },
    },
    ripple: true,
  });
  app.directive("tooltip", Tooltip);
}
