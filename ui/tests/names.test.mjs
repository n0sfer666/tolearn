import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import test, { before } from "node:test";
import { fileURLToPath } from "node:url";
import { createContext, runInContext } from "node:vm";

import { DIST } from "../scripts/budget.mjs";
import { browser } from "./support/dom.mjs";

const UI = fileURLToPath(new URL("..", import.meta.url));
const PROGRAM = "nes-dev";
const TITLE = "Разработка под NES";

before(
  () => {
    execFileSync("pnpm", ["exec", "astro", "build"], { cwd: UI, stdio: "inherit" });
  },
  { timeout: 300_000 },
);

const INLINE = /<script(?![^>]*\b(?:src|type)=)[^>]*>([\s\S]*?)<\/script>/g;

function fillers(html) {
  const found = [...html.matchAll(INLINE)]
    .map(([, code]) => code)
    .filter((code) => code.includes("tolearn."));
  if (!found.some((code) => code.includes("tolearn.names"))) {
    throw new Error("на странице нет скрипта имён");
  }
  return found;
}

function visit(route, search, names = null) {
  const html = readFileSync(path.join(DIST, route, "index.html"), "utf8");
  const window = browser(`https://tolearn.local/${route}/${search}`);
  window.localStorage.clear();
  if (names !== null) window.localStorage.setItem("tolearn.names", JSON.stringify(names));
  window.document.body.innerHTML = html.match(/<header[\s\S]*?<\/header>/)[0];
  const context = createContext({
    document: window.document,
    location: window.location,
    localStorage: window.localStorage,
    addEventListener: window.addEventListener.bind(window),
    URLSearchParams: globalThis.URLSearchParams,
    URL: globalThis.URL,
    JSON: globalThis.JSON,
  });
  for (const code of fillers(html)) runInContext(code, context);
  return window;
}

const KNOWN = { program: { id: PROGRAM, title: TITLE } };

test("шапка программы зовёт программу по имени", () => {
  const { document } = visit("ru/program", `?program=${PROGRAM}`, KNOWN);

  assert.equal(document.querySelector(".bar-title").textContent, TITLE);
  assert.equal(document.querySelector(".bar-title").getAttribute("title"), TITLE);
});

test("шапка этапа зовёт программу по имени в ссылке назад", () => {
  const { document } = visit("ru/stage", `?program=${PROGRAM}&node=rom&stage=first-rom`, KNOWN);

  assert.equal(document.querySelector("[data-back]").textContent, TITLE);
  assert.equal(document.querySelector(".bar-title").textContent, "Этап");
});

test("имя чужой программы в шапку не попадает", () => {
  const { document } = visit("ru/program", "?program=snes-dev", KNOWN);

  assert.equal(document.querySelector(".bar-title").textContent, "Программа");
});

test("название приезжает островом, когда экран уже открыт", () => {
  const window = visit("ru/program", `?program=${encodeURIComponent(PROGRAM)}`);

  assert.equal(window.document.querySelector(".bar-title").textContent, "Программа");

  window.dispatchEvent(
    new window.CustomEvent("tolearn:name", {
      detail: { kind: "program", id: PROGRAM, title: TITLE },
    }),
  );

  assert.equal(window.document.querySelector(".bar-title").textContent, TITLE);
});

test("экран без выбранной программы остаётся при своём заголовке", () => {
  const { document } = visit("ru/settings", "", KNOWN);

  assert.equal(document.querySelector(".bar-title").textContent, "Настройки");
});
