import assert from "node:assert/strict";
import test, { before } from "node:test";

import { island } from "../scripts/island.mjs";
import { en } from "../src/i18n/en.ts";
import { ru } from "../src/i18n/ru.ts";
import { browser, settled } from "./support/dom.mjs";

let Program;
let render;
let window;

before(
  async () => {
    window = browser("https://tolearn.local/ru/program/?program=nes-dev&node=rom");
    globalThis.location = window.location;
    ({ default: Program } = await island("Program"));
    ({ render } = await import("solid-js/web"));
  },
  { timeout: 300_000 },
);

const span = (min, max) => ({ min, max });

const ROOT = {
  program: "nes-dev",
  uuid: "nes-dev",
  title: "Разработка игр для NES",
  goal: "Собрать игру для NES и запустить её в эмуляторе",
  level: "Начинающий: знает Python",
  hours: span(14, 22),
  trail: [],
  stages: [
    { id: "setup", title: "Окружение", hours: span(2, 3), ready: true },
    { id: "sprites", title: "Спрайты", hours: span(3, 4), ready: false },
  ],
  children: [
    { id: "tools", title: "Инструменты сборки", hours: span(8, 12), ready: true },
    { id: "sound", title: "Звук", hours: span(6, 10), ready: false },
  ],
};

const NESTED = {
  ...ROOT,
  uuid: "rom",
  title: "Первый ROM",
  trail: [
    { uuid: "nes-dev", title: "Разработка игр для NES" },
    { uuid: "tools", title: "Инструменты сборки" },
  ],
  children: [],
};

function mount(options = {}) {
  const host = window.document.createElement("div");
  window.document.body.append(host);
  const calls = [];
  const call = (name, payload) => {
    calls.push({ name, payload });
    if (name !== "node") throw new Error(`лишняя команда ${name}`);
    return options.fail ? Promise.reject(options.fail) : Promise.resolve(options.out ?? ROOT);
  };
  const names = [];
  window.addEventListener("tolearn:name", (event) => names.push(event.detail));
  const props = { text: ru, locale: "ru", call, program: "nes-dev", node: "", ...options.props };
  render(() => Program(props), host);
  return { host, calls, names };
}

test("без выбранной программы экран говорит об этом, а не пустеет", async () => {
  const { host, calls } = mount({ props: { program: "" } });
  await settled();

  assert.equal(calls.length, 0, "ядро дёрнули без программы");
  assert.match(host.querySelector("[data-empty]").textContent, /Программа не выбрана/);
  assert.equal(host.querySelector("[data-empty] a").getAttribute("href"), "/ru/");
});

test("экран читает узел по программе и подпрограмме", async () => {
  const { calls } = mount({ props: { node: "tools" } });
  await settled();

  assert.deepEqual(calls, [{ name: "node", payload: { program: "nes-dev", node: "tools" } }]);
});

test("без явных параметров программа и узел берутся из адреса", async () => {
  const { calls } = mount({ props: { program: undefined, node: undefined } });
  await settled();

  assert.deepEqual(calls[0].payload, { program: "nes-dev", node: "rom" });
});

test("цель, уровень и часы карты видны", async () => {
  const { host } = mount();
  await settled();

  assert.match(host.querySelector("[data-goal]").textContent, /Собрать игру для NES/);
  assert.match(host.querySelector("[data-level]").textContent, /Начинающий/);
  assert.match(host.querySelector("[data-node-hours]").textContent, /14\D+22/);
});

test("сгенерированный этап ведёт на свой экран, остальные помечены", async () => {
  const { host } = mount();
  await settled();

  const ready = host.querySelector('[data-stage="setup"]');
  const pending = host.querySelector('[data-stage="sprites"]');
  assert.equal(ready.querySelector("a").getAttribute("href"), "/ru/stage/?program=nes-dev&stage=setup");
  assert.match(ready.textContent, /2\D+3/);
  assert.equal(pending.querySelector("a"), null, "несгенерированный этап остался ссылкой");
  assert.equal(pending.dataset.pending, "");
  assert.match(pending.textContent, new RegExp(ru.program.pending));
});

test("готовая подпрограмма ведёт в свой узел, несгенерированная помечена", async () => {
  const { host } = mount();
  await settled();

  const tools = host.querySelector('[data-child="tools"]');
  const sound = host.querySelector('[data-child="sound"]');
  assert.equal(tools.querySelector("a").getAttribute("href"), "/ru/program/?program=nes-dev&node=tools");
  assert.equal(sound.querySelector("a"), null);
  assert.match(sound.textContent, new RegExp(ru.program.pending));
});

test("вложенный узел показывает путь, и Esc уводит к родителю", async () => {
  const { host } = mount({ out: NESTED, props: { node: "rom" } });
  await settled();

  const crumbs = [...host.querySelectorAll("[data-trail] a")];
  assert.deepEqual(
    crumbs.map((crumb) => crumb.getAttribute("href")),
    ["/ru/program/?program=nes-dev", "/ru/program/?program=nes-dev&node=tools"],
  );
  assert.equal(host.querySelector("[data-up]"), crumbs[1]);
  assert.equal(host.querySelector("[data-node-title]").textContent, "Первый ROM");
  assert.equal(
    host.querySelector('[data-stage="setup"] a').getAttribute("href"),
    "/ru/stage/?program=nes-dev&node=rom&stage=setup",
  );
});

test("у корня пути нет — вверх ведёт ссылка назад в шапке", async () => {
  const { host } = mount();
  await settled();

  assert.equal(host.querySelector("[data-trail]"), null);
  assert.equal(host.querySelector("[data-up]"), null);
});

test("в шапку уходит название всей программы, а не подпрограммы", async () => {
  const root = mount();
  await settled();
  assert.deepEqual(root.names.at(-1), { kind: "program", id: "nes-dev", title: ROOT.title });

  const nested = mount({ out: NESTED, props: { node: "rom" } });
  await settled();
  assert.deepEqual(nested.names.at(-1), { kind: "program", id: "nes-dev", title: "Разработка игр для NES" });
});

test("неоткрывшийся узел называет причину на языке словаря и ведёт в библиотеку", async () => {
  const absent = { code: "node.absent", message: "подпрограммы `sound` в программе нет" };
  const { host } = mount({ fail: absent });
  const english = mount({ fail: absent, props: { text: en, locale: "en" } });
  const gone = mount({ fail: { code: "library.absent", message: "the library holds no program" } });
  const strange = mount({ fail: { code: "library.unreadable", message: "`programs` не читается" } });
  await settled();

  const reason = (root) => root.querySelector("[data-empty]").textContent;
  assert.ok(reason(host).includes(ru.program.nodeAbsent), reason(host));
  assert.equal(host.querySelector("[data-empty] a").getAttribute("href"), "/ru/");
  assert.ok(reason(english.host).includes(en.program.nodeAbsent), reason(english.host));
  assert.equal(english.host.querySelector("[data-empty] a").getAttribute("href"), "/en/");
  assert.ok(reason(gone.host).includes(ru.program.absent), reason(gone.host));
  assert.ok(reason(strange.host).includes("`programs` не читается"), reason(strange.host));
});

test("у узла без этапов нет пустого заголовка этапов", async () => {
  const { host } = mount({ out: { ...ROOT, stages: [] } });
  await settled();

  assert.equal(host.querySelector("[data-stages]"), null);
  assert.ok(![...host.querySelectorAll("h2")].some((head) => head.textContent === ru.program.stages));
});
