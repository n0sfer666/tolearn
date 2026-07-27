import solid from "@astrojs/solid-js";
import { defineConfig } from "astro/config";

export default defineConfig({
  output: "static",
  build: { format: "directory", inlineStylesheets: "auto" },
  integrations: [solid()],
  devToolbar: { enabled: false },
  prefetch: false,
  vite: { build: { assetsInlineLimit: 0 } },
});
