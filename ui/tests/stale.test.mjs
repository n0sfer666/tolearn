import assert from "node:assert/strict";
import test, { before } from "node:test";

import { island } from "../scripts/island.mjs";
import { ru } from "../src/i18n/ru.ts";
import { browser, settled } from "./support/dom.mjs";

let Stale;
let render;
let document;

before(
  async () => {
    ({ document } = browser("https://tolearn.local/ru/stale/?program=/programs/llm-agents-base"));
    ({ default: Stale } = await island("Stale"));
    ({ render } = await import("solid-js/web"));
  },
  { timeout: 300_000 },
);

const OUT = {
  topics: [
    {
      topic: "local-runtime",
      title: "Локальный рантайм",
      verified_at: "2026-05-01",
      expired_at: "2026-05-31",
    },
  ],
  materials: [
    {
      topic: "structured-output",
      topic_title: "Машиночитаемый вывод",
      title: "Structured Outputs Intro",
      url: "https://example.invalid/structured",
      stale: true,
      delta: "Перечень моделей устарел, механику читать можно.",
      covers_version: "gpt-4o-2024-08-06",
      pin: "unknown",
    },
    {
      topic: "provider-routing",
      topic_title: "Шлюз к провайдерам",
      title: "LiteLLM release notes",
      url: "https://example.invalid/litellm",
      stale: false,
      delta: null,
      covers_version: "v1.93.0",
      pin: "known",
    },
  ],
};

function mount(options = {}) {
  const host = document.createElement("div");
  document.body.append(host);
  const call = (name) => {
    if (name === "stale") return Promise.resolve({ ...OUT, ...options.out });
    throw new Error(`лишняя команда ${name}`);
  };
  const dispose = render(
    () =>
      Stale({
        text: ru,
        locale: "ru",
        path: "/programs/llm-agents-base",
        today: "2026-07-28",
        call,
      }),
    host,
  );
  return { host, dispose };
}

test("устаревшая тема названа обеими датами", async () => {
  const { host } = mount();
  await settled();

  const item = host.querySelector("[data-expired='local-runtime']");
  assert.ok(item, host.innerHTML);
  assert.match(item.querySelector("[data-verified]").textContent, /2026-05-01/);
  assert.match(item.querySelector("[data-since]").textContent, /2026-05-31/);
});

test("из строки темы есть ход в саму тему", async () => {
  const { host } = mount();
  await settled();

  const link = host.querySelector("[data-expired='local-runtime'] a");
  assert.ok(link, host.innerHTML);
  assert.match(link.getAttribute("href"), /\/ru\/topic\/\?program=/);
  assert.match(link.getAttribute("href"), /topic=local-runtime/);
});

test("расхождение показано текстом, а не флагом", async () => {
  const { host } = mount();
  await settled();

  const delta = host.querySelector("[data-delta]");
  assert.ok(delta, host.innerHTML);
  assert.match(delta.textContent, /Перечень моделей устарел/);
});

test("неизвестный пин назван вместе с заявленной версией", async () => {
  const { host } = mount();
  await settled();

  const pin = host.querySelector("[data-pin]");
  assert.ok(pin, host.innerHTML);
  assert.match(pin.textContent, new RegExp(ru.stale.pinUnknown));
  assert.match(pin.textContent, /gpt-4o-2024-08-06/);
});

test("про совпавший пин ничего не выдумано", async () => {
  const { host } = mount();
  await settled();

  const known = host.querySelector("[data-aging='LiteLLM release notes']");
  assert.ok(known, host.innerHTML);
  assert.equal(known.querySelector("[data-pin]"), null, known.innerHTML);
  assert.equal(known.querySelector("[data-flagged]"), null, known.innerHTML);
});

test("пустой дайджест говорит, что всё свежее", async () => {
  const { host } = mount({ out: { topics: [], materials: [] } });
  await settled();

  assert.ok(host.querySelector("[data-empty]"), host.innerHTML);
  assert.equal(host.querySelector("[data-topics]"), null, host.innerHTML);
});
