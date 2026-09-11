import path from "node:path";
import { fileURLToPath } from "node:url";
import { build } from "vite";
import solid from "vite-plugin-solid";

const UI = fileURLToPath(new URL("..", import.meta.url));
const OUT = path.join(UI, "node_modules/.islands");

async function compile(entry, file, ssr) {
  await build({
    root: UI,
    logLevel: "error",
    plugins: [solid({ ssr })],
    build: {
      outDir: OUT,
      emptyOutDir: false,
      lib: { entry, formats: ["es"], fileName: () => file },
      rollupOptions: { external: ["solid-js", "solid-js/web", "solid-js/store"] },
    },
  });
  return import(path.join(OUT, file));
}

export function island(name, from = "src/islands", ext = "tsx") {
  return compile(path.join(UI, from, `${name}.${ext}`), `${name}.mjs`, false);
}

export function hydratable(name) {
  return compile(path.join(UI, "src/islands", `${name}.tsx`), `${name}.hydratable.mjs`, true);
}
