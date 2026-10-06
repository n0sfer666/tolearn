import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test, { before } from "node:test";

import { blocks, declarations } from "../scripts/css.mjs";
import { island } from "../scripts/island.mjs";
import { parts } from "../scripts/targets.mjs";
import { styles } from "../scripts/tokens.mjs";
import { en } from "../src/i18n/en.ts";
import { ru } from "../src/i18n/ru.ts";
import { browser, settled } from "./support/dom.mjs";

let Program;
let render;
let window;

before(
  async () => {
    window = browser("https://tolearn.local/ru/program/?program=nes-dev");
    globalThis.location = window.location;
    ({ default: Program } = await island("Program"));
    ({ render } = await import("solid-js/web"));
  },
  { timeout: 300_000 },
);

const span = (min, max) => ({ min, max });
const stage = (id, status, ready = true) => ({
  id,
  title: id,
  hours: span(1, 2),
  ready,
  status,
  pass: status === "passed" ? "exam" : null,
});

const NONE = { books: [], pages: [] };

const NODE = {
  program: "nes-dev",
  uuid: "nes-dev",
  title: "Разработка игр для NES",
  goal: "Собрать игру для NES",
  level: "Начинающий",
  hours: span(14, 22),
  trail: [],
  stages: [stage("setup", "passed"), stage("loop", "opened"), stage("input", "fresh"), stage("sprites", "fresh", false)],
  children: [{ id: "tools", title: "Инструменты", hours: span(8, 12), ready: true, summary: null }],
  summary: { passed: 1, total: 4, skipped: 0 },
  sources: NONE,
};

const SOURCES = {
  books: [{ title: "Game Sound", authors: ["Karen Collins"], chapter: "2. Push Start Button" }],
  pages: [{ title: "NESdev Wiki: APU", url: "https://www.nesdev.org/wiki/APU", checked_at: "2026-09-11" }],
};

function mount(out, props = {}) {
  const host = window.document.createElement("div");
  window.document.body.append(host);
  const call = () => Promise.resolve(out);
  render(() => Program({ text: ru, locale: "ru", call, program: "nes-dev", node: "", ...props }), host);
  return host;
}

test("«вы здесь» и «Продолжить» стоят у первого сгенерированного этапа, который не пройден", async () => {
  const host = mount(NODE);
  await settled();

  const marked = [...host.querySelectorAll("[data-here]")].map((mark) => mark.closest("[data-stage]").dataset.stage);
  assert.deepEqual(marked, ["loop"]);
  assert.equal(host.querySelector("[data-here]").textContent, ru.program.here);
  const resume = host.querySelector('[data-stage="loop"] [data-resume]');
  assert.equal(resume.getAttribute("href"), "/ru/stage/?program=nes-dev&stage=loop");
  assert.equal(resume.textContent, ru.program.resume);
  assert.ok(resume.hasAttribute("data-action"), "«Продолжить» не видно в ⌘K");
});

test("когда сгенерированное пройдено, отметки нет — несгенерированный этап её не получает", async () => {
  const host = mount({ ...NODE, stages: [stage("setup", "passed"), stage("sprites", "fresh", false)] });
  await settled();

  assert.equal(host.querySelectorAll("[data-here]").length, 0);
  assert.equal(host.querySelectorAll("[data-resume]").length, 0);
});

test("этапы и подпрограммы идут маршрутом", async () => {
  const host = mount(NODE);
  await settled();

  assert.equal(host.querySelector("[data-stages]").dataset.route, "");
  assert.equal(host.querySelector("[data-children]").dataset.route, "");
});

test("источники узла: книга с авторами и главой, страница ссылкой и датой проверки", async () => {
  const host = mount({ ...NODE, sources: SOURCES });
  const english = mount({ ...NODE, sources: SOURCES }, { text: en, locale: "en" });
  await settled();

  const book = host.querySelector("[data-sources] [data-book]");
  assert.equal(book.textContent, `Karen Collins. Game Sound, ${ru.program.chapter} 2. Push Start Button`);
  assert.equal(book.querySelector("cite").textContent, "Game Sound");
  const page = host.querySelector("[data-sources] [data-page]");
  assert.equal(page.querySelector("a").getAttribute("href"), "https://www.nesdev.org/wiki/APU");
  assert.equal(page.querySelector("a").textContent, "NESdev Wiki: APU");
  assert.equal(page.querySelector("time").getAttribute("datetime"), "2026-09-11");
  assert.equal(page.querySelector("[data-checked]").textContent, `${ru.program.checked} 11 сентября 2026 г.`);
  assert.equal(english.querySelector("[data-checked]").textContent, `${en.program.checked} September 11, 2026`);
});

test("у узла без источников блока источников нет", async () => {
  const host = mount(NODE);
  await settled();

  assert.equal(host.querySelector("[data-sources]"), null);
});

async function route() {
  const found = [];
  for (const file of await styles()) found.push(...blocks(readFileSync(file, "utf8")));
  return found
    .filter(({ selector }) => parts(selector).some((part) => part.includes("[data-route]")))
    .flatMap(({ body }) => declarations(body));
}

test("длинный маршрут прокручивается вбок, а узлы не сжимаются", async () => {
  const found = await route();
  const last = (property) => found.filter((one) => one.property === property).at(-1)?.value;

  assert.equal(last("overflow-x"), "auto");
  assert.equal(last("grid-auto-flow"), "column");
  assert.match(last("grid-auto-columns") ?? "", /^minmax\(var\(--[\w-]+\)/);
});
