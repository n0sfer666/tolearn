import assert from "node:assert/strict";
import test, { before } from "node:test";

import { island } from "../scripts/island.mjs";
import { ru } from "../src/i18n/ru.ts";
import { browser, settled } from "./support/dom.mjs";

let Graph;
let render;
let document;

before(
  async () => {
    ({ document } = browser("https://tolearn.local/ru/graph/?program=/programs/llm-agents-base"));
    ({ default: Graph } = await island("Graph"));
    ({ render } = await import("solid-js/web"));
  },
  { timeout: 300_000 },
);

const OUT = {
  nodes: [
    {
      id: "local-runtime",
      title: "Локальный рантайм",
      status: "passed",
      layer: 0,
      depends_on: [],
      blocked_by: [],
      unlocks: ["openai-compatible-api", "structured-output"],
    },
    {
      id: "openai-compatible-api",
      title: "OpenAI-совместимый API",
      status: "todo",
      layer: 1,
      depends_on: ["local-runtime"],
      blocked_by: [],
      unlocks: ["structured-output"],
    },
    {
      id: "structured-output",
      title: "Машиночитаемый вывод",
      status: "blocked",
      layer: 2,
      depends_on: ["local-runtime", "openai-compatible-api"],
      blocked_by: ["openai-compatible-api"],
      unlocks: [],
    },
  ],
};

function mount(out = {}) {
  const host = document.createElement("div");
  document.body.append(host);
  const call = (name) => {
    if (name === "graph") return Promise.resolve({ ...OUT, ...out });
    throw new Error(`лишняя команда ${name}`);
  };
  render(
    () =>
      Graph({
        text: ru,
        locale: "ru",
        path: "/programs/llm-agents-base",
        today: "2026-07-28",
        call,
      }),
    host,
  );
  return host;
}

test("узлы разложены по слоям в порядке зависимостей", async () => {
  const host = mount();
  await settled();

  const layers = [...host.querySelectorAll("[data-layer]")].map((layer) =>
    layer.getAttribute("data-layer"),
  );

  assert.deepEqual(layers, ["0", "1", "2"]);
  assert.ok(host.querySelector("[data-layer='0'] [data-node='local-runtime']"), host.innerHTML);
  assert.ok(host.querySelector("[data-layer='2'] [data-node='structured-output']"), host.innerHTML);
});

test("узел кликабелен и ведёт в свою тему", async () => {
  const host = mount();
  await settled();

  const link = host.querySelector("[data-node='structured-output'] [data-open]");

  assert.match(link.getAttribute("href"), /\/ru\/topic\/\?program=/);
  assert.match(link.getAttribute("href"), /topic=structured-output/);
});

test("заблокированная тема названа чем разблокируется", async () => {
  const host = mount();
  await settled();

  const waiting = host.querySelector("[data-node='structured-output'] [data-waiting]");

  assert.ok(waiting, host.innerHTML);
  assert.match(waiting.textContent, /OpenAI-совместимый API/);
  assert.equal(host.querySelector("[data-node='local-runtime'] [data-waiting]"), null);
});

test("тема говорит, что разблокирует", async () => {
  const host = mount();
  await settled();

  const unlocks = host.querySelector("[data-node='local-runtime'] [data-unlocks]");

  assert.match(unlocks.textContent, /OpenAI-совместимый API/);
  assert.match(unlocks.textContent, /Машиночитаемый вывод/);
  assert.ok(host.querySelector("[data-node='structured-output'] [data-leaf]"), host.innerHTML);
});

test("статус узла читается словом, а не цветом", async () => {
  const host = mount();
  await settled();

  const blocked = host.querySelector("[data-node='structured-output'] [data-state]");
  const passed = host.querySelector("[data-node='local-runtime'] [data-state]");

  assert.equal(blocked.textContent, ru.status.blocked);
  assert.equal(passed.textContent, ru.status.passed);
  assert.notEqual(blocked.textContent, passed.textContent);
});

test("пустая программа сказана словами", async () => {
  const host = mount({ nodes: [] });
  await settled();

  assert.ok(host.querySelector("[data-empty]"), host.innerHTML);
});
