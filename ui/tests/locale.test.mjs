import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import test, { before } from "node:test";
import { fileURLToPath } from "node:url";

import { DIST, scripts, weight } from "../scripts/budget.mjs";
import { island } from "../scripts/island.mjs";
import { browser } from "./support/dom.mjs";

const UI = fileURLToPath(new URL("..", import.meta.url));

const DEFAULTS = {
  disk_budget_mb: 2048,
  notes_directory: null,
  locale: "ru",
  theme: "system",
  history_depth: 5,
  history_share_percent: 10,
};

let chosen;
let mirrored;
let boot;
let window;

before(
  async () => {
    window = browser("https://tolearn.local/");
    ({ chosen, mirrored } = await island("locale", "src/lib", "ts"));
    ({ boot } = await island("boot", "src/lib", "ts"));
    execFileSync("pnpm", ["exec", "astro", "build"], { cwd: UI, stdio: "inherit" });
  },
  { timeout: 300_000 },
);

function core(view) {
  return (name, payload) => {
    if (name !== "settings" || payload.save !== null) throw new Error(`лишний запрос ${name}`);
    if (view === null) return Promise.reject(new Error("ядро молчит"));
    return Promise.resolve({ ...DEFAULTS, ...view });
  };
}

function source(route) {
  const file = path.join(DIST, route, "index.html");
  const html = readFileSync(file, "utf8");
  return [html, ...weight(file).files.map((name) => readFileSync(path.join(DIST, name), "utf8"))]
    .join("\n");
}

test("сохранённый язык важнее зеркала и системного", () => {
  assert.equal(chosen("en", "ru", "ru-RU"), "en");
  assert.equal(chosen("ru", "en", "en-US"), "ru");
});

test("зеркало отвечает, только пока ядро молчит", () => {
  assert.equal(chosen(null, "en", "ru-RU"), "en");
  assert.equal(chosen(null, null, "en-US"), "en");
});

test("незнакомый язык откатывается на русский", () => {
  assert.equal(chosen("de", null, "de-DE"), "ru");
  assert.equal(chosen(null, "кириллица", "zz"), "ru");
});

test("старт спрашивает ядро и чинит разошедшееся зеркало", async () => {
  window.localStorage.setItem("tolearn.locale", "en");

  assert.equal(await boot(core({ locale: "ru" })), "ru");
  assert.equal(mirrored(), "ru");
});

test("молчащее ядро не роняет старт — идём по зеркалу", async () => {
  window.localStorage.setItem("tolearn.locale", "en");

  assert.equal(await boot(core(null)), "en");
  assert.equal(mirrored(), "en");
});

test("старт возвращает и тему из ядра, а не только язык", async () => {
  window.localStorage.removeItem("tolearn.theme");

  await boot(core({ locale: "ru", theme: "dark" }));

  assert.equal(window.localStorage.getItem("tolearn.theme"), "dark");
});

test("стартовая страница спрашивает ядро о языке", () => {
  assert.match(source(""), /"settings"/, "стартовый код не обращается к ядру");
});

test("стартовая страница не решает язык до ответа ядра", () => {
  for (const code of scripts(readFileSync(path.join(DIST, "index.html"), "utf8")).inline) {
    assert.doesNotMatch(code, /location\.replace/, "переход решён встроенным скриптом, без ядра");
  }
});
