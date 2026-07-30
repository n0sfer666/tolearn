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

before(() => {
  execFileSync("pnpm", ["exec", "astro", "build"], { cwd: UI, stdio: "inherit" });
}, { timeout: 300_000 });

function page(route) {
  return readFileSync(path.join(DIST, route, "index.html"), "utf8");
}

function keeper(html) {
  const found = [...html.matchAll(/<script(?![^>]*\bsrc=)[^>]*>([\s\S]*?)<\/script>/g)]
    .map(([, code]) => code)
    .filter((code) => code.includes("data-keep"));
  if (found.length !== 1) throw new Error(`ожидался один скрипт контекста, найдено ${found.length}`);
  return found[0];
}

function visit(route, search, remembered = null) {
  const html = page(route);
  const window = browser(`https://tolearn.local/${route}/${search}`);
  window.localStorage.clear();
  if (remembered !== null) window.localStorage.setItem("tolearn.program", remembered);
  window.document.body.innerHTML = html.match(/<header[\s\S]*?<\/header>/)[0];
  runInNewContext(keeper(html), {
    document: window.document,
    location: window.location,
    localStorage: window.localStorage,
    URL: globalThis.URL,
    URLSearchParams: globalThis.URLSearchParams,
  });
  return window.document;
}

const OPEN = "?program=%2Fprograms%2Fllm-agents-base&topic=cp-gateway";

test("ссылка назад уносит открытую программу с собой", () => {
  const document = visit("ru/topic", OPEN);

  assert.equal(
    document.querySelector("[data-back]").getAttribute("href"),
    "/ru/program/?program=%2Fprograms%2Fllm-agents-base",
  );
});

test("назад с экрана практики держит и программу, и тему", () => {
  const document = visit("ru/practice", OPEN);

  assert.equal(
    document.querySelector("[data-back]").getAttribute("href"),
    "/ru/topic/?program=%2Fprograms%2Fllm-agents-base&topic=cp-gateway",
  );
});

test("поиск в шапке открывается в той же программе", () => {
  const document = visit("ru/topic", OPEN);

  assert.equal(
    document.querySelector("[data-search]").getAttribute("href"),
    "/ru/search/?program=%2Fprograms%2Fllm-agents-base",
  );
});

test("разделы вне программы контекст не тянут", () => {
  const document = visit("ru/topic", OPEN);

  assert.equal(document.querySelector("[data-queue]").getAttribute("href"), "/ru/queue/");
  assert.equal(document.querySelector("[data-settings]").getAttribute("href"), "/ru/settings/");
});

test("шапка отмечает раздел, в котором стоишь", () => {
  const queue = visit("ru/queue", "");

  assert.equal(queue.querySelector("[data-queue]").getAttribute("aria-current"), "page");
  assert.equal(queue.querySelector("[data-search]").getAttribute("aria-current"), null);
  assert.equal(visit("ru/topic", OPEN).querySelectorAll("[aria-current]").length, 0);
});

test("без контекста в адресе ссылки остаются как были", () => {
  const document = visit("ru/topic", "");

  assert.equal(document.querySelector("[data-back]").getAttribute("href"), "/ru/program/");
  assert.equal(document.querySelector("[data-search]").getAttribute("href"), "/ru/search/");
});

test("открытая программа запоминается на будущее", () => {
  const document = visit("ru/topic", OPEN);

  assert.equal(
    document.defaultView.localStorage.getItem("tolearn.program"),
    "/programs/llm-agents-base",
  );
});

test("поиск из шапки открывает последнюю программу, когда адрес её не несёт", () => {
  const document = visit("ru/queue", "", "/programs/llm-agents-base");

  assert.equal(
    document.querySelector("[data-search]").getAttribute("href"),
    "/ru/search/?program=%2Fprograms%2Fllm-agents-base",
  );
  assert.equal(document.querySelector("[data-settings]").getAttribute("href"), "/ru/settings/");
});
