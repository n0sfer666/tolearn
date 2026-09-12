import { spawnSync } from "node:child_process";
import { mkdtempSync, readdirSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const here = resolve(import.meta.dirname, "..", "..");
const programs = [
  join(here, "examples", "chiptune"),
  join(here, "fixtures", "v2", "valid", "nes-dev"),
];

const room = mkdtempSync(join(tmpdir(), "tolearn-markdown-"));

const ran = (command, args, options = {}) =>
  spawnSync(command, args, { stdio: "inherit", cwd: here, ...options });

const pages = (folder) =>
  readdirSync(folder, { recursive: true })
    .filter((name) => name.endsWith(".md"))
    .map((name) => join(folder, name));

try {
  const written = programs.flatMap((program, place) => {
    const into = join(room, String(place));
    const built = ran("cargo", ["run", "--quiet", "-p", "tolearn-cli", "--", "export", program, into]);
    if (built.status !== 0) {
      throw new Error(`экспорт ${program} не собрался`);
    }
    return pages(into);
  });

  const linted = ran(
    "pnpm",
    ["exec", "markdownlint-cli2", "--config", join(here, ".markdownlint-cli2.jsonc"), ...written],
    { cwd: resolve(import.meta.dirname, "..") },
  );
  if (linted.status !== 0) {
    throw new Error("экспорт не прошёл markdownlint");
  }
  console.log(`markdown: ${written.length} страниц экспорта прошли markdownlint`);
} finally {
  rmSync(room, { recursive: true, force: true });
}
