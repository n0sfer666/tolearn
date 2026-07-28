import { spawnSync } from "node:child_process";
import { cpSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const here = resolve(import.meta.dirname, "..", "..");
const bundle = join(here, "examples", "llm-agents-base");

const room = mkdtempSync(join(tmpdir(), "tolearn-markdown-"));
const copy = join(room, "bundle");
const exported = join(room, "program.md");

const ran = (command, args, options = {}) =>
  spawnSync(command, args, { stdio: "inherit", cwd: here, ...options });

try {
  cpSync(bundle, copy, {
    recursive: true,
    filter: (source) => !source.endsWith(".json"),
  });

  const built = ran("cargo", [
    "run",
    "--quiet",
    "-p",
    "tolearn-cli",
    "--",
    "export",
    copy,
    "--out",
    exported,
    "--today",
    "2026-07-29",
  ]);
  if (built.status !== 0) {
    throw new Error("экспорт не собрался");
  }

  const linted = ran(
    "pnpm",
    ["exec", "markdownlint-cli2", "--config", join(here, ".markdownlint-cli2.jsonc"), exported],
    { cwd: resolve(import.meta.dirname, "..") },
  );
  if (linted.status !== 0) {
    throw new Error("экспорт не прошёл markdownlint");
  }
  console.log("markdown: экспорт эталонного бандла прошёл markdownlint");
} finally {
  rmSync(room, { recursive: true, force: true });
}
