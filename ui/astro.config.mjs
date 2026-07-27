import { fileURLToPath } from "node:url";

import solid from "@astrojs/solid-js";
import { defineConfig } from "astro/config";

const design = fileURLToPath(new URL("../docs/design", import.meta.url));

export default defineConfig({
  output: "static",
  build: { format: "directory", inlineStylesheets: "auto" },
  integrations: [solid()],
  devToolbar: { enabled: false },
  prefetch: false,
  vite: {
    build: { assetsInlineLimit: 0 },
    resolve: { alias: { "@design": design } },
    server: { fs: { allow: [design, fileURLToPath(new URL(".", import.meta.url))] } },
  },
});
