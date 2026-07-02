import { createApp } from "vue";
import { registerPlugins } from "@/app/plugins";
import { router } from "@/app/router";
import App from "@/App.vue";

import "primeicons/primeicons.css";
import "./styles.css";

const app = createApp(App);
registerPlugins(app);
app.use(router);
app.mount("#app");
