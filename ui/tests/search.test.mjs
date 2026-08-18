import assert from "node:assert/strict";
import test, { before } from "node:test";

import { island } from "../scripts/island.mjs";
import { ru } from "../src/i18n/ru.ts";
import { browser, settled, toasts } from "./support/dom.mjs";

let Search;
let render;
let document;

before(
  async () => {
    ({ document } = browser("https://tolearn.local/ru/search/?program=/bundle"));
    ({ default: Search } = await island("Search"));
    ({ render } = await import("solid-js/web"));
  },
  { timeout: 300_000 },
);

const HIT = {
  kind: "topic",
  roadmap: "llm-agents-base",
  topic: "local-runtime",
  title: "Локальный рантайм",
  snippet: "поднять модель",
};

function mount(options = {}) {
  const host = document.createElement("div");
  document.body.append(host);
  const calls = [];
  const call = (name, payload) => {
    calls.push({ name, payload });
    if (options.refuse === true) {
      return Promise.reject({ code: "search.bundle", message: "нет" });
    }
    return Promise.resolve({ hits: options.hits ?? [HIT], indexed: 0 });
  };
  const said = toasts(document.defaultView);
  render(() => Search({ text: ru, locale: "ru", program: options.program ?? "/bundle", call }), host);
  return { host, calls, said };
}

function ask(host, text) {
  const input = host.querySelector("[data-query]");
  input.value = text;
  input.dispatchEvent(new host.ownerDocument.defaultView.Event("input", { bubbles: true }));
  host.querySelector("[data-find]").click();
}

test("экран ничего не ищет до запроса", async () => {
  const { host, calls } = mount();
  await settled();

  assert.equal(calls.length, 0);
  assert.equal(host.querySelectorAll("[data-hit]").length, 0);
});

test("запрос уходит в ядро вместе с программой", async () => {
  const { host, calls } = mount();
  ask(host, "рантайм");
  await settled();

  assert.deepEqual(calls[0], {
    name: "search",
    payload: { bundle: "/bundle", query: "рантайм", directory: null, limit: 20 },
  });
});

test("находка ведёт на экран темы", async () => {
  const { host } = mount();
  ask(host, "рантайм");
  await settled();

  const hit = host.querySelector("[data-hit]");
  assert.equal(hit.getAttribute("data-hit"), "topic");
  assert.equal(hit.querySelector("a").getAttribute("href"), "/ru/topic/?program=%2Fbundle&topic=local-runtime");
  assert.equal(hit.querySelector("[data-kind]").textContent, ru.search.topic);
  assert.equal(hit.querySelector("[data-snippet]").textContent, "поднять модель");
});

test("находка в конспекте ведёт на экран конспекта", async () => {
  const { host } = mount({ hits: [{ ...HIT, kind: "note", title: "local-runtime" }] });
  ask(host, "рантайм");
  await settled();

  const hit = host.querySelector("[data-hit]");
  assert.equal(hit.getAttribute("data-hit"), "note");
  assert.equal(hit.querySelector("a").getAttribute("href"), "/ru/notes/?program=%2Fbundle&topic=local-runtime");
  assert.equal(hit.querySelector("[data-kind]").textContent, ru.search.note);
});

test("пустая выдача говорит об этом", async () => {
  const { host } = mount({ hits: [] });
  ask(host, "тарабарщина");
  await settled();

  assert.equal(host.querySelector("[data-nothing]").textContent, ru.search.nothing);
});

test("отказ ядра показан, а не проглочен", async () => {
  const { host, said } = mount({ refuse: true });
  ask(host, "рантайм");
  await settled();

  assert.deepEqual(said.at(-1), { tone: "error", text: ru.search.failed });
  assert.equal(host.querySelectorAll("[data-hit]").length, 0);
  assert.equal(host.querySelector("[data-nothing]"), null, "отказ выдан за пустую выдачу");
});

test("без открытой программы экран зовёт выбрать её, а не ищет вслепую", async () => {
  const { host, calls } = mount({ program: "" });
  await settled();

  assert.equal(calls.length, 0);
  assert.equal(host.querySelector("[data-query]"), null);
  assert.match(host.querySelector("[data-empty]").textContent, new RegExp(ru.program.none));
});
