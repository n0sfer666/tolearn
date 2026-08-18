import assert from "node:assert/strict";
import test, { before, mock } from "node:test";

import { island } from "../scripts/island.mjs";
import { ru } from "../src/i18n/ru.ts";
import { toast as say } from "../src/lib/toast.ts";
import { browser, settled } from "./support/dom.mjs";

let Toasts;
let render;
let document;
let window;

before(
  async () => {
    window = browser();
    document = window.document;
    ({ default: Toasts } = await island("Toasts", "src/components"));
    ({ render } = await import("solid-js/web"));
  },
  { timeout: 300_000 },
);

function mount() {
  const host = document.createElement("div");
  document.body.append(host);
  const dispose = render(() => Toasts({ close: ru.toast.close }), host);
  return { host, dispose };
}

const rows = (host) => [...host.querySelectorAll("[data-toast]")];

test("сообщение приходит наверх с тоном, который ему дали", async () => {
  const { host, dispose } = mount();
  await settled();

  say("ok", "сохранено");
  await settled();

  const [row] = rows(host);
  assert.equal(row.getAttribute("data-toast"), "ok");
  assert.equal(row.querySelector("[data-toast-text]").textContent, "сохранено");
  assert.equal(row.getAttribute("role"), "status");
  dispose();
});

test("ошибка и предупреждение объявляются громко, спокойные тона — нет", async () => {
  const { host, dispose } = mount();
  await settled();

  say("error", "не сохранено");
  say("warn", "осторожно");
  say("info", "к сведению");
  await settled();

  assert.deepEqual(
    rows(host).map((row) => [row.getAttribute("data-toast"), row.getAttribute("role")]),
    [
      ["error", "alert"],
      ["warn", "alert"],
      ["info", "status"],
    ],
  );
  dispose();
});

test("сообщение закрывается рукой", async () => {
  const { host, dispose } = mount();
  await settled();

  say("info", "к сведению");
  await settled();
  host.querySelector("[data-toast-close]").click();
  await settled();

  assert.deepEqual(rows(host), []);
  dispose();
});

test("сообщение уходит само, ошибка висит дольше остальных", async () => {
  mock.timers.enable({ apis: ["setTimeout"] });
  const { host, dispose } = mount();

  say("ok", "сохранено");
  say("error", "не сохранено");
  mock.timers.tick(0);

  assert.equal(rows(host).length, 2);

  mock.timers.tick(3000);
  assert.deepEqual(
    rows(host).map((row) => row.getAttribute("data-toast")),
    ["error"],
  );

  mock.timers.tick(6000);
  assert.deepEqual(rows(host), []);

  dispose();
  mock.timers.reset();
});

test("пустой текст экран не засоряет", async () => {
  const { host, dispose } = mount();
  await settled();

  say("ok", "");
  say("выдуманный", "тон не из списка");
  await settled();

  assert.deepEqual(rows(host), []);
  dispose();
});
