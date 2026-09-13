import assert from "node:assert/strict";
import test, { before } from "node:test";

import { island } from "../scripts/island.mjs";
import { en } from "../src/i18n/en.ts";
import { ru } from "../src/i18n/ru.ts";
import { browser, settled, toasts } from "./support/dom.mjs";

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

const NODE = {
  program: "nes-dev",
  uuid: "rom",
  title: "Первый ROM",
  goal: "Собрать ROM и запустить его",
  level: "Начинающий",
  hours: span(3, 5),
  trail: [{ uuid: "nes-dev", title: "Разработка игр для NES" }],
  stages: [{ id: "first-rom", title: "Первый ROM", hours: span(1, 2), ready: true, status: "fresh", pass: null }],
  children: [],
  summary: { passed: 0, total: 1, skipped: 0 },
};

const FOLDER = "/Users/me/Выгрузка";
const WRITTEN = `${FOLDER}/first-rom`;

function mount(options = {}) {
  const host = window.document.createElement("div");
  window.document.body.append(host);
  const calls = [];
  const call = (name, payload) => {
    calls.push({ name, payload });
    if (name === "node") return Promise.resolve(NODE);
    if (name !== "export") throw new Error(`лишняя команда ${name}`);
    return options.fail ? Promise.reject(options.fail) : Promise.resolve({ path: WRITTEN, files: 5 });
  };
  const pick = () => Promise.resolve(options.folder === undefined ? FOLDER : options.folder);
  const said = toasts(window);
  const props = { text: ru, locale: "ru", call, program: "nes-dev", node: "", pick, ...options.props };
  render(() => Program(props), host);
  return { host, calls, said };
}

const exports = (calls) => calls.filter((one) => one.name === "export");

async function pressed(options = {}) {
  const mounted = mount(options);
  await settled();
  mounted.host.querySelector("[data-export]").click();
  await settled();
  return mounted;
}

test("кнопка экспорта подписана словарём", async () => {
  const { host } = mount();
  await settled();

  assert.equal(host.querySelector("[data-export]").textContent, ru.program.export);
});

test("экспорт выгружает показанный узел в выбранную папку", async () => {
  const { calls, said } = await pressed();

  assert.deepEqual(exports(calls), [
    { name: "export", payload: { program: "nes-dev", node: "rom", folder: FOLDER } },
  ]);
  assert.deepEqual(said, [{ tone: "ok", text: `${ru.program.exported} ${WRITTEN}` }]);
});

test("отменённый выбор папки ничего не выгружает", async () => {
  const { calls, said } = await pressed({ folder: null });

  assert.deepEqual(exports(calls), []);
  assert.deepEqual(said, []);
});

test("пока экспорт идёт, второе нажатие не запускает второй", async () => {
  const { host, calls } = mount();
  await settled();

  const button = host.querySelector("[data-export]");
  button.click();
  assert.equal(button.getAttribute("aria-disabled"), "true");
  button.click();
  await settled();

  assert.equal(exports(calls).length, 1);
  assert.equal(button.getAttribute("aria-disabled"), "false");
});

test("отказ назван словарём, неизвестный — сообщением ядра", async () => {
  const occupied = await pressed({ fail: { code: "export.occupied", message: "`x` is taken" } });
  assert.deepEqual(occupied.said.at(-1), { tone: "error", text: ru.program.occupied });

  const library = await pressed({
    fail: { code: "export.inside-library", message: "`x` лежит в библиотеке" },
    props: { text: en, locale: "en" },
  });
  assert.deepEqual(library.said.at(-1), { tone: "error", text: en.program.insideLibrary });

  const folder = await pressed({ fail: { code: "export.folder", message: "`x` — не абсолютный путь" } });
  assert.deepEqual(folder.said.at(-1), { tone: "error", text: ru.program.badFolder });

  const strange = await pressed({ fail: { code: "export.strange", message: "`x` сломался" } });
  assert.deepEqual(strange.said.at(-1), { tone: "error", text: "`x` сломался" });

  const mute = await pressed({ fail: new Error("") });
  assert.deepEqual(mute.said.at(-1), { tone: "error", text: ru.program.exportFailed });
});
