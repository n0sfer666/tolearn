import path from "node:path";
import { fileURLToPath } from "node:url";
import { build } from "vite";
import solid from "vite-plugin-solid";

const UI = fileURLToPath(new URL("..", import.meta.url));
const OUT = path.join(UI, "node_modules/.islands");

export async function island(name, from = "src/islands") {
  await build({
    root: UI,
    logLevel: "error",
    plugins: [solid()],
    build: {
      outDir: OUT,
      emptyOutDir: false,
      lib: {
        entry: path.join(UI, from, `${name}.tsx`),
        formats: ["es"],
        fileName: () => `${name}.mjs`,
      },
      rollupOptions: { external: ["solid-js", "solid-js/web", "solid-js/store"] },
    },
  });
  return import(path.join(OUT, `${name}.mjs`));
}
