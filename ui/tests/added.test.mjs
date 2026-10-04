import assert from "node:assert/strict";
import test from "node:test";

import { en } from "../src/i18n/en.ts";
import { ru } from "../src/i18n/ru.ts";
import { settled } from "./support/dom.mjs";
import { OUT, stageScreen } from "./support/stage.mjs";

const { mount } = stageScreen();

const ADDED = {
  ...OUT,
  questions: [
    { ...OUT.questions[0], result: "ok", answer: "Пять каналов.", added: "Сэмплы `DPCM` короткие." },
    { ...OUT.questions[1], result: "ok", answer: "Форма волны." },
  ],
};

test("дополнение стоит под вопросом тем же Rich, что эталон", async () => {
  const { host } = mount({ out: ADDED });
  await settled();

  const block = host.querySelector("#q1 [data-added]");
  assert.equal(block.querySelector("[data-added-title]").textContent, ru.stage.added);
  assert.equal(block.querySelector("code").textContent, "DPCM");
  assert.match(block.textContent, /Сэмплы DPCM короткие\./);
});

test("без дополнения блока нет", async () => {
  const fresh = mount();
  const graded = mount({ out: ADDED });
  await settled();

  assert.equal(fresh.host.querySelector("[data-added]"), null);
  assert.equal(graded.host.querySelector("#q2 [data-added]"), null);
  assert.ok(graded.host.querySelector("#q2 [data-mirror]"));
});

test("заголовок дополнения говорит на языке словаря", async () => {
  const { host } = mount({ out: ADDED, text: en });
  await settled();

  assert.equal(host.querySelector("#q1 [data-added-title]").textContent, en.stage.added);
});
