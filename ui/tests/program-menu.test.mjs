import assert from "node:assert/strict";
import test, { before } from "node:test";

import { island } from "../scripts/island.mjs";
import { en } from "../src/i18n/en.ts";
import { ru } from "../src/i18n/ru.ts";
import { browser, settled, toasts } from "./support/dom.mjs";

let Menu;
let render;
let window;

before(
  async () => {
    window = browser("https://tolearn.local/ru/program/?program=nes-dev&node=rom");
    globalThis.location = window.location;
    ({ default: Menu } = await island("ProgramMenu"));
    ({ render } = await import("solid-js/web"));
  },
  { timeout: 300_000 },
);

const FOLDER = "/Users/me/Выгрузка";
const WRITTEN = `${FOLDER}/first-rom`;

function mount(options = {}) {
  const host = window.document.createElement("div");
  window.document.body.append(host);
  const calls = [];
  const call = (name, payload) => {
    calls.push({ name, payload });
    if (name !== "export") throw new Error(`лишняя команда ${name}`);
    return options.fail ? Promise.reject(options.fail) : Promise.resolve({ path: WRITTEN, files: 5 });
  };
  const pick = () => Promise.resolve(options.folder === undefined ? FOLDER : options.folder);
  const said = toasts(window);
  render(() => Menu({ text: ru, call, pick, ...options.props }), host);
  return { host, calls, said };
}

async function pressed(options = {}) {
  const mounted = mount(options);
  await settled();
  mounted.host.querySelector("[data-export]").click();
  await settled();
  return mounted;
}

test("меню программы открывается кнопкой, экспорт лежит внутри и виден в ⌘K", () => {
  const { host } = mount();

  const menu = host.querySelector("details[data-menu]");
  assert.equal(menu.querySelector(":scope > summary").getAttribute("aria-label"), ru.program.menu);
  assert.equal(menu.open, false);
  const item = menu.querySelector("[data-export]");
  assert.equal(item.textContent, ru.program.export);
  assert.ok(item.hasAttribute("data-action"));
});

test("экспорт выгружает узел из адреса в выбранную папку", async () => {
  const { calls, said } = await pressed();

  assert.deepEqual(calls, [{ name: "export", payload: { program: "nes-dev", node: "rom", folder: FOLDER } }]);
  assert.deepEqual(said, [{ tone: "ok", text: `${ru.program.exported} ${WRITTEN}` }]);
});

test("у корня узел пустой — выгружается вся программа", async () => {
  const { calls } = await pressed({ props: { program: "chiptune", node: "" } });

  assert.deepEqual(calls[0].payload, { program: "chiptune", node: "", folder: FOLDER });
});

test("выбранный пункт закрывает меню", async () => {
  const { host } = mount();
  const menu = host.querySelector("details[data-menu]");
  menu.open = true;

  menu.querySelector("[data-export]").click();
  await settled();

  assert.equal(menu.open, false);
});

test("отменённый выбор папки ничего не выгружает", async () => {
  const { calls, said } = await pressed({ folder: null });

  assert.deepEqual(calls, []);
  assert.deepEqual(said, []);
});

test("пока экспорт идёт, второе нажатие не запускает второй", async () => {
  const { host, calls } = mount();

  const button = host.querySelector("[data-export]");
  button.click();
  assert.equal(button.getAttribute("aria-disabled"), "true");
  button.click();
  await settled();

  assert.equal(calls.length, 1);
  assert.equal(button.getAttribute("aria-disabled"), "false");
});

test("отказ назван словарём, неизвестный — сообщением ядра", async () => {
  const occupied = await pressed({ fail: { code: "export.occupied", message: "`x` is taken" } });
  assert.deepEqual(occupied.said.at(-1), { tone: "error", text: ru.program.occupied });

  const library = await pressed({
    fail: { code: "export.inside-library", message: "`x` лежит в библиотеке" },
    props: { text: en },
  });
  assert.deepEqual(library.said.at(-1), { tone: "error", text: en.program.insideLibrary });

  const strange = await pressed({ fail: { code: "export.strange", message: "`x` сломался" } });
  assert.deepEqual(strange.said.at(-1), { tone: "error", text: "`x` сломался" });

  const mute = await pressed({ fail: new Error("") });
  assert.deepEqual(mute.said.at(-1), { tone: "error", text: ru.program.exportFailed });
});
