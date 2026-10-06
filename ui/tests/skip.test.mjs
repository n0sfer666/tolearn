import assert from "node:assert/strict";
import test from "node:test";

import { ru } from "../src/i18n/ru.ts";
import { settled, toasts } from "./support/dom.mjs";
import { named, press } from "./support/generation.mjs";
import { stageScreen } from "./support/stage.mjs";

const { screen, mount } = stageScreen();

const PLACE = { program: "chip", node: "chip", stage: "voices" };
const NEXT = "/ru/next/?program=chip&stage=voices";

function skipping(reply) {
  const went = [];
  const said = toasts(screen.window);
  const mounted = mount({ replies: { skip: reply }, props: { go: (href) => went.push(href) } });
  return { ...mounted, went, said };
}

test("«Пропустить зачёт» записывает пропуск этапа и ведёт на развилку", async () => {
  const { host, calls, went } = skipping(() => ({ skipped: true }));
  await settled();

  press(host, "[data-next]");
  await settled();

  assert.deepEqual(
    named(calls, "skip").map((made) => made.payload),
    [PLACE],
  );
  assert.deepEqual(went, [NEXT]);
});

test("на уже пройденном этапе пропуск ничего не пишет, но на развилку ведёт", async () => {
  const { host, went } = skipping(() => ({ skipped: false }));
  await settled();

  press(host, "[data-next]");
  await settled();

  assert.deepEqual(went, [NEXT]);
});

test("двойное нажатие шлёт один пропуск", async () => {
  let release = () => {};
  const pending = new Promise((resolve) => {
    release = resolve;
  });
  const { host, calls, went } = skipping(() => pending);
  await settled();

  press(host, "[data-next]");
  press(host, "[data-next]");
  release({ skipped: true });
  await settled();
  await settled();

  assert.equal(named(calls, "skip").length, 1);
  assert.deepEqual(went, [NEXT]);
});

test("пропуск не записался — тост, этап остаётся открытым", async () => {
  const { host, went, said } = skipping(() => {
    throw new Error("диск полон");
  });
  await settled();

  press(host, "[data-next]");
  await settled();

  assert.deepEqual(went, []);
  assert.deepEqual(said.at(-1), { tone: "error", text: ru.stage.unskipped });
  assert.equal(host.querySelector("[data-next]").getAttribute("aria-busy"), "false");
});

test("«К развилке» ведёт на развилку и ничего не пишет", async () => {
  const { host, calls, went } = skipping(() => ({ skipped: true }));
  await settled();

  press(host, "[data-fork]");
  await settled();

  assert.deepEqual(named(calls, "skip"), []);
  assert.deepEqual(went, [NEXT]);
  assert.equal(host.querySelector("[data-fork]").textContent, ru.generate.fork);
});
