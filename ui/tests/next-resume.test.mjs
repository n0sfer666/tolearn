import assert from "node:assert/strict";
import test from "node:test";

import { ru } from "../src/i18n/ru.ts";
import { settled } from "./support/dom.mjs";
import { QUIET, ending, job, named, press, refusal, sequence } from "./support/generation.mjs";
import { FORK, forkScreen } from "./support/next.mjs";

const { mount } = forkScreen();
const hint = (host) => host.querySelector("[data-progress] [data-leave]")?.textContent ?? null;
const answered = (...states) => sequence(...states.map((state) => () => Promise.resolve(state)));
const back = (...states) => mount({ answers: { generation_state: answered(...states) } });
const settle = async () => {
  for (let turn = 0; turn < 4; turn += 1) await settled();
};

test("идущая развилка этого узла показывает ход и fork второй раз не зовёт", async () => {
  const fork = job("fork");
  const done = { work: null, outcome: ending(fork, { fork: FORK }) };
  const { host, calls, tell } = back({ work: fork, outcome: null }, done);
  await settle();

  assert.equal(hint(host), ru.generate.leaveFork);
  assert.equal(named(calls, "fork").length, 0, "экран открыл развилку второй раз");

  tell(done);
  await settle();
  assert.ok(host.querySelector('[data-variant="noise"]') !== null, "готовые варианты не встали");
  assert.equal(named(calls, "generation_seen").length, 1, "увиденные варианты остались в шапке");
});

test("идущий следующий этап этого узла показывает ход и открывает этап, когда тот готов", async () => {
  const next = job("next");
  const { host, calls, gone, tell } = back({ work: next, outcome: null });
  await settle();

  assert.equal(hint(host), ru.generate.leaveStage);
  assert.equal(named(calls, "fork").length, 0);
  tell({ work: null, outcome: ending(next, { stage: { program: "chip", node: "rom", stage: "dpcm" } }) });
  await settle();
  assert.deepEqual(gone, ["/ru/stage/?program=chip&node=rom&stage=dpcm"]);
});

test("варианты, готовые без экрана, встают на нём и гасятся", async () => {
  const done = { work: null, outcome: ending(job("fork"), { fork: FORK }) };
  const { host, calls } = back(done);
  await settle();

  assert.ok(host.querySelector('[data-variant="dpcm"]') !== null);
  assert.equal(named(calls, "fork").length, 0);
  assert.equal(named(calls, "generation_seen").length, 1);
});

test("работа над чужим узлом или этапом развилку не подменяет", async () => {
  for (const other of [{ node: "noise" }, { stage: "pulse" }]) {
    const { host, calls } = back({ work: job("fork", other), outcome: null }, QUIET);
    await settle();

    assert.equal(named(calls, "fork").length, 1, JSON.stringify(other));
    assert.ok(host.querySelector('[data-variant="noise"]') !== null);
  }
});

test("следующий этап, готовый без экрана, открывается сразу, развилку заново не просят", async () => {
  const built = { program: "chip", node: "rom", stage: "dpcm" };
  const { calls, gone } = back({ work: null, outcome: ending(job("next"), { stage: built, seen: true }) });
  await settle();

  assert.deepEqual(gone, ["/ru/stage/?program=chip&node=rom&stage=dpcm"]);
  assert.equal(named(calls, "fork").length, 0, "экран открыл развилку второй раз");
});

test("отмена на вернувшемся экране зовёт cancel_generation, тост — только у нажавшего", async () => {
  const next = job("next");
  const halted = { work: null, outcome: ending(next, { refusal: refusal("generate.cancelled", "отменено") }) };
  const pressed = back({ work: next, outcome: null });
  const idle = back({ work: next, outcome: null });
  await settle();

  press(pressed.host, "[data-cancel]");
  await settle();
  assert.equal(named(pressed.calls, "cancel_generation").length, 1);
  pressed.tell(halted);
  idle.tell(halted);
  await settle();
  assert.deepEqual(pressed.said.at(-1), { tone: "info", text: ru.generate.cancelled });
  assert.equal(pressed.said.filter((seen) => seen.text === ru.generate.cancelled).length, 1, "тост от второго экрана");
});
