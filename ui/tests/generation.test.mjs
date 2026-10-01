import assert from "node:assert/strict";
import test, { after, before } from "node:test";

import { island } from "../scripts/island.mjs";
import { ru } from "../src/i18n/ru.ts";
import { BUILDING, PLANNING } from "../src/lib/jobs.ts";
import { browser, settled, toasts } from "./support/dom.mjs";
import { deferred, heard, job, named, refusal, transport } from "./support/generation.mjs";

const alive = [];
const CANCELLED = refusal("generate.cancelled", "отменено");
let generation;
let createRoot;
let said;

before(
  async () => {
    const window = browser("https://tolearn.local/ru/");
    said = toasts(window);
    ({ generation } = await island("generation", "src/lib", "ts"));
    ({ createRoot } = await import("solid-js"));
  },
  { timeout: 300_000 },
);

after(() => {
  for (const dispose of alive) dispose();
});

function followed(answers = {}) {
  const { calls, call } = transport({ cancel_generation: { cancelled: true }, ...answers });
  const work = createRoot((dispose) => {
    alive.push(dispose);
    return generation(ru, () => call, heard().steps, false);
  });
  return { calls, work };
}

const cancelled = () => said.filter((seen) => seen.text === ru.generate.cancelled).length;

test("отмена работы, которую остров не запускал, зовёт cancel_generation и говорит «Генерация отменена»", async () => {
  const { calls, work } = followed();
  const drawn = deferred();
  const was = cancelled();
  const ended = work.run(drawn.answer, PLANNING, job("plan"));

  work.cancel();
  await settled();
  assert.equal(named(calls, "cancel_generation").length, 1);
  assert.ok(work.cancelling(), "нет «Останавливаю…»");

  drawn.reject(CANCELLED);
  assert.equal(await ended, null);
  assert.equal(work.ended(), "halted");
  assert.equal(cancelled() - was, 1);
});

test("чужая отмена без нажатия гасит ход молча", async () => {
  const { calls, work } = followed();
  const drawn = deferred();
  const was = cancelled();
  const ended = work.run(drawn.answer, BUILDING, job("next"));

  drawn.reject(CANCELLED);
  assert.equal(await ended, null);
  assert.equal(work.ended(), "halted");
  assert.equal(named(calls, "cancel_generation").length, 0);
  assert.equal(cancelled() - was, 0, "тост у острова, который не отменял");
});

test("опоздавшая отмена чужого этапа говорит generate.late, ход идёт дальше", async () => {
  const { work } = followed({ cancel_generation: { cancelled: false } });
  const drawn = deferred();
  void work.run(drawn.answer, BUILDING, job("next"));

  work.cancel();
  await settled();
  assert.deepEqual(said.at(-1), { tone: "info", text: ru.generate.late });
  assert.ok(work.running());
  assert.ok(!work.cancelling());
  drawn.resolve({ program: "chip", node: "rom", stage: "dpcm" });
});
