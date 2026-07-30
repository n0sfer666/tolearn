import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import test, { before } from "node:test";
import { fileURLToPath } from "node:url";
import { runInNewContext } from "node:vm";

import { DIST } from "../scripts/budget.mjs";
import { browser } from "./support/dom.mjs";

const UI = fileURLToPath(new URL("..", import.meta.url));
const PROGRAM = "/programs/llm-agents-base";
const TITLE = "Работа с агентами LLM — база на готовых инструментах";
const TOPIC = "Локальный рантайм — поднять модель и понять, что съело память";

before(
  () => {
    execFileSync("pnpm", ["exec", "astro", "build"], { cwd: UI, stdio: "inherit" });
  },
  { timeout: 300_000 },
);

function filler(html) {
  const found = [...html.matchAll(/<script(?![^>]*\bsrc=)[^>]*>([\s\S]*?)<\/script>/g)]
    .map(([, code]) => code)
    .filter((code) => code.includes("tolearn.names"));
  if (found.length !== 1) throw new Error(`ожидался один скрипт имён, найдено ${found.length}`);
  return found[0];
}

function visit(route, search, names = null) {
  const html = readFileSync(path.join(DIST, route, "index.html"), "utf8");
  const window = browser(`https://tolearn.local/${route}/${search}`);
  window.localStorage.clear();
  if (names !== null) window.localStorage.setItem("tolearn.names", JSON.stringify(names));
  window.document.body.innerHTML = html.match(/<header[\s\S]*?<\/header>/)[0];
  runInNewContext(filler(html), {
    document: window.document,
    location: window.location,
    localStorage: window.localStorage,
    addEventListener: window.addEventListener.bind(window),
    URLSearchParams: globalThis.URLSearchParams,
    JSON: globalThis.JSON,
  });
  return window;
}

const KNOWN = {
  program: { id: PROGRAM, title: TITLE },
  topic: { id: "local-runtime", title: TOPIC },
};

test("шапка темы зовёт тему по имени, а «назад» — программу", () => {
  const { document } = visit(
    "ru/topic",
    `?program=${encodeURIComponent(PROGRAM)}&topic=local-runtime`,
    KNOWN,
  );

  assert.equal(document.querySelector(".bar-title").textContent, TOPIC);
  assert.equal(document.querySelector("[data-back]").textContent, TITLE);
  assert.equal(document.querySelector(".bar-title").getAttribute("title"), TOPIC);
});

test("имя чужой темы в шапку не попадает", () => {
  const { document } = visit(
    "ru/topic",
    `?program=${encodeURIComponent(PROGRAM)}&topic=cp-gateway`,
    KNOWN,
  );

  assert.equal(document.querySelector(".bar-title").textContent, "Тема");
  assert.equal(document.querySelector("[data-back]").textContent, TITLE);
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
  const { document } = visit("ru/queue", "", KNOWN);

  assert.equal(document.querySelector(".bar-title").textContent, "Повторение");
});
