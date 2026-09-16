import assert from "node:assert/strict";
import test, { before, beforeEach } from "node:test";

import { island } from "../scripts/island.mjs";
import { en } from "../src/i18n/en.ts";
import { ru } from "../src/i18n/ru.ts";
import { browser, settled, toasts } from "./support/dom.mjs";

let Menu;
let render;
let window;

before(
  async () => {
    window = browser("https://tolearn.local/ru/program/?program=nes-dev");
    globalThis.location = window.location;
    ({ default: Menu } = await island("ProgramMenu"));
    ({ render } = await import("solid-js/web"));
  },
  { timeout: 300_000 },
);

const TITLE = "Разработка игр для NES";

beforeEach(() => {
  window.document.body.innerHTML = "";
  window.localStorage.clear();
});

function mount(options = {}) {
  const host = window.document.createElement("div");
  window.document.body.append(host);
  const calls = [];
  const gone = [];
  const call = (name, payload) => {
    calls.push({ name, payload });
    if (name === "node") return Promise.resolve({ title: TITLE, trail: [] });
    if (name !== "delete_program") throw new Error(`лишняя команда ${name}`);
    return options.fail ? Promise.reject(options.fail) : Promise.resolve({ title: TITLE });
  };
  const said = toasts(window);
  render(
    () =>
      Menu({
        text: ru,
        locale: "ru",
        call,
        pick: () => Promise.resolve(null),
        go: (href) => gone.push(href),
        titled: () => TITLE,
        ...options.props,
      }),
    host,
  );
  return { host, calls, said, gone };
}

function layer() {
  return window.document.querySelector("[data-confirm]");
}

async function asked(options = {}) {
  const mounted = mount(options);
  mounted.host.querySelector("[data-delete]").click();
  await settled();
  return mounted;
}

function press(key, init = {}) {
  layer().dispatchEvent(new window.KeyboardEvent("keydown", { key, bubbles: true, cancelable: true, ...init }));
}

test("пункт удаления лежит в меню программы и виден в ⌘K", () => {
  const { host } = mount();

  const item = host.querySelector("[data-delete]");
  assert.equal(item.textContent, ru.program.delete);
  assert.ok(item.hasAttribute("data-action"));
  assert.equal(layer(), null);
});

test("у подпрограммы своего удаления нет", () => {
  const { host } = mount({ props: { node: "rom" } });

  assert.equal(host.querySelector("[data-delete]"), null);
  assert.ok(host.querySelector("[data-export]"));
});

test("подтверждение называет программу, что уходит и что вернётся", async () => {
  await asked();

  const ask = layer();
  assert.equal(ask.getAttribute("role"), "dialog");
  assert.equal(ask.getAttribute("aria-modal"), "true");
  const named = ask.querySelector(`#${ask.getAttribute("aria-labelledby")}`);
  assert.equal(named.textContent, `Удалить «${TITLE}»?`);
  assert.equal(ask.querySelectorAll("p")[0].textContent, ru.program.deleteGoes);
  assert.equal(ask.querySelectorAll("p")[1].textContent, ru.program.deleteRestore);
});

test("без своего источника название берётся из кэша имён экрана", async () => {
  window.localStorage.setItem(
    "tolearn.names",
    JSON.stringify({ program: { id: "nes-dev", title: TITLE } }),
  );

  const { calls } = await asked({ props: { titled: undefined } });

  assert.equal(layer().querySelector("h2").textContent, `Удалить «${TITLE}»?`);
  assert.deepEqual(calls, []);
});

test("кэш пуст или от другой программы — название спрашивается у ядра", async () => {
  window.localStorage.setItem(
    "tolearn.names",
    JSON.stringify({ program: { id: "tools", title: "Инструменты" } }),
  );

  const { calls } = await asked({ props: { titled: undefined } });
  await settled();

  assert.deepEqual(calls, [{ name: "node", payload: { program: "nes-dev", node: "" } }]);
  assert.equal(layer().querySelector("h2").textContent, `Удалить «${TITLE}»?`);
});

test("фокус садится на отмену и запирается внутри слоя", async () => {
  await asked();

  const no = layer().querySelector("[data-confirm-no]");
  const yes = layer().querySelector("[data-confirm-yes]");
  assert.equal(window.document.activeElement, no);
  press("Tab");
  assert.equal(window.document.activeElement, yes);
  press("Tab");
  assert.equal(window.document.activeElement, no);
  press("Tab", { shiftKey: true });
  assert.equal(window.document.activeElement, yes);
});

test("Esc возвращает фокус в меню и ничего не удаляет", async () => {
  const { calls, said, host } = await asked();

  press("Escape");
  await settled();

  assert.equal(window.document.activeElement, host.querySelector("summary"));
  assert.equal(layer(), null);
  assert.deepEqual(calls, []);
  assert.deepEqual(said, []);
});

test("экранные шорткаты не выходят за слой подтверждения", async () => {
  await asked();
  const heard = [];
  window.document.addEventListener("keydown", (event) => heard.push(event.key));

  press("k", { metaKey: true });
  press("/");

  assert.deepEqual(heard, []);
});

test("Esc закрывает подтверждение и ничего не удаляет", async () => {
  const { calls, said } = await asked();

  press("Escape");
  await settled();

  assert.equal(layer(), null);
  assert.deepEqual(calls, []);
  assert.deepEqual(said, []);
});

test("отмена закрывает подтверждение и ничего не удаляет", async () => {
  const { calls } = await asked();

  layer().querySelector("[data-confirm-no]").click();
  await settled();

  assert.equal(layer(), null);
  assert.deepEqual(calls, []);
});

test("подтверждённое удаление зовёт команду и уходит в библиотеку с именем удалённого", async () => {
  const { calls, said, gone } = await asked();

  layer().querySelector("[data-confirm-yes]").click();
  await settled();

  assert.deepEqual(calls, [{ name: "delete_program", payload: { program: "nes-dev" } }]);
  assert.deepEqual(said, []);
  assert.deepEqual(gone, [`/ru/?${new URLSearchParams({ deleted: TITLE })}`]);
  assert.equal(layer(), null);
});

test("отказ назван словарём, неизвестный — сообщением ядра", async () => {
  const cases = [
    ["delete.busy", ru.program.deleteBusy],
    ["delete.subprogram", ru.program.deleteSubprogram],
    ["delete.cache", ru.program.deleteCache],
    ["delete.broken", ru.program.deleteBroken],
    ["library.absent", ru.program.deleteGone],
  ];
  for (const [code, text] of cases) {
    const { said, gone } = await asked({ fail: { code, message: "ядро сказало" } });
    layer().querySelector("[data-confirm-yes]").click();
    await settled();
    assert.deepEqual(said.at(-1), { tone: "error", text });
    assert.deepEqual(gone, []);
  }

  const strange = await asked({ fail: { code: "delete.strange", message: "корзина ушла" } });
  layer().querySelector("[data-confirm-yes]").click();
  await settled();
  assert.deepEqual(strange.said.at(-1), { tone: "error", text: "корзина ушла" });

  const mute = await asked({ fail: new Error(""), props: { text: en } });
  layer().querySelector("[data-confirm-yes]").click();
  await settled();
  assert.deepEqual(mute.said.at(-1), { tone: "error", text: en.program.deleteFailed });
});

test("сбой корзины называет то, что осталось на месте", async () => {
  const left = "корзина не приняла: programs/nes-dev (нет прав), state/nes-dev";
  const { said, gone } = await asked({ fail: { code: "delete.left", message: left } });

  layer().querySelector("[data-confirm-yes]").click();
  await settled();

  assert.deepEqual(said.at(-1), { tone: "error", text: `${ru.program.deleteLeft} ${left}` });
  assert.deepEqual(gone, []);
});
