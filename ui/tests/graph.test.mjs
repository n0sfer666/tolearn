import assert from "node:assert/strict";
import test, { before } from "node:test";

import { island } from "../scripts/island.mjs";
import { laid, links } from "../src/components/graph/layout.ts";
import { at, near } from "../src/components/graph/paint.ts";
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

test("карта нарисована холстом, а список убирается кнопкой", async () => {
  const host = mount();
  await settled();

  const map = host.querySelector("[data-map]");
  const toggle = host.querySelector("[data-list-toggle]");

  assert.equal(map.getAttribute("aria-label"), ru.graph.map);
  assert.ok(host.querySelector("[data-graph]"), host.innerHTML);
  toggle.click();
  await settled();
  assert.equal(host.querySelector("[data-graph]"), null);
  assert.equal(host.querySelector("[data-list-toggle]").getAttribute("aria-expanded"), "false");
});

test("линии карты идут только между темами этой программы", () => {
  const drawn = links([
    ...OUT.nodes,
    { ...OUT.nodes[1], id: "чужая", depends_on: ["из-другой-программы"] },
  ]);

  assert.deepEqual(
    drawn.filter((link) => link.to === "structured-output").map((link) => link.from),
    ["local-runtime", "openai-compatible-api"],
  );
  assert.equal(
    drawn.some((link) => link.from === "из-другой-программы"),
    false,
  );
});

test("раскладка разводит темы и повторяется от прогона к прогону", () => {
  const spots = laid(OUT.nodes);
  const again = laid(OUT.nodes);

  const here = spots.get("local-runtime");
  const there = spots.get("structured-output");
  assert.ok(Math.hypot(here.x - there.x, here.y - there.y) > 40, JSON.stringify([here, there]));
  assert.deepEqual([...again.entries()], [...spots.entries()]);
});

test("клик по точке попадает в её тему, а мимо — ни в какую", () => {
  const spots = laid(OUT.nodes);
  const view = { scale: 1, x: 0, y: 0 };
  const spot = spots.get("openai-compatible-api");

  assert.equal(near(OUT.nodes, spots, 1, at(view, spot.x, spot.y)), "openai-compatible-api");
  assert.equal(near(OUT.nodes, spots, 1, at(view, spot.x + 400, spot.y + 400)), "");
});
