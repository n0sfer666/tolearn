import assert from "node:assert/strict";
import test, { before } from "node:test";

import { island } from "../scripts/island.mjs";
import { ru } from "../src/i18n/ru.ts";
import { browser, settled } from "./support/dom.mjs";

let Review;
let render;
let document;

before(
  async () => {
    ({ document } = browser(
      "https://tolearn.local/ru/review/?program=/programs/llm-agents-base&topic=local-runtime",
    ));
    ({ default: Review } = await island("Review"));
    ({ render } = await import("solid-js/web"));
  },
  { timeout: 300_000 },
);

const OUT = {
  id: "local-runtime",
  title: "Локальный рантайм",
  questions: [
    { id: "q1", kind: "diagnose", text: "Почему модель занимает больше памяти?", outcome: "miss", missed: ["квантование"] },
    { id: "q2", kind: "predict", text: "Что будет с окном контекста?", outcome: null, missed: [] },
  ],
  loose: [],
  last: { at: "2026-07-28T10:00:00+03:00", verdict: "fail" },
  history: [{ at: "2026-07-27T10:00:00+03:00", verdict: "fail" }],
  split_suggested: false,
  split_request: "",
};

function mount(patch = {}, options = {}) {
  const host = document.createElement("div");
  document.body.append(host);
  const calls = [];
  const call = (name, payload) => {
    calls.push({ name, payload });
    if (name !== "review") throw new Error(`лишняя команда ${name}`);
    return Promise.resolve({ ...OUT, ...patch });
  };
  const copied = [];
  render(
    () =>
      Review({
        text: ru,
        program: "/programs/llm-agents-base",
        topic: "local-runtime",
        call,
        copy: (text) => {
          if (options.noClipboard === true) return Promise.reject(new Error("нет буфера"));
          copied.push(text);
          return Promise.resolve();
        },
      }),
    host,
  );
  return { host, calls, copied };
}

test("пробел висит на том вопросе, который его упустил", async () => {
  const { host, calls } = mount();
  await settled();

  assert.deepEqual(calls[0], {
    name: "review",
    payload: { bundle: "/programs/llm-agents-base", topic: "local-runtime" },
  });
  const first = host.querySelector('[data-question="q1"]');
  assert.match(first.querySelector("[data-missed]").textContent, /квантование/);
  assert.match(first.querySelector("[data-outcome]").textContent, new RegExp(ru.review.resultMiss));
});

test("вопрос без ответа показан без исхода", async () => {
  const { host } = mount();
  await settled();

  const second = host.querySelector('[data-question="q2"]');
  assert.equal(second.querySelector("[data-outcome]"), null);
  assert.equal(second.querySelector("[data-missed]"), null);
});

test("ничей пробел показан отдельно", async () => {
  const { host } = mount({ loose: ["общая картина"] });
  await settled();

  assert.match(host.querySelector("[data-loose]").textContent, /общая картина/);
});

test("последняя попытка подробно, прошлые — строкой", async () => {
  const { host } = mount();
  await settled();

  assert.match(host.querySelector("[data-last]").textContent, /2026-07-28/);
  assert.equal(host.querySelectorAll("[data-history] li").length, 1);
  assert.match(host.querySelector("[data-history]").textContent, new RegExp(ru.review.verdictFail));
});

test("без попыток разбор говорит об этом", async () => {
  const { host } = mount({ last: null, history: [] });
  await settled();

  assert.match(host.querySelector("[data-none]").textContent, new RegExp(ru.review.none));
  assert.equal(host.querySelector("[data-history]"), null);
});

test("три провала подряд дают текст запроса и копируют его одним действием", async () => {
  const { host, copied } = mount({ split_suggested: true, split_request: "Раздели тему `local-runtime`" });
  await settled();

  assert.match(host.querySelector("[data-request]").textContent, /local-runtime/);
  host.querySelector("[data-copy]").click();
  await settled();

  assert.deepEqual(copied, ["Раздели тему `local-runtime`"]);
  assert.match(host.textContent, new RegExp(ru.review.copied));
});

test("без буфера обмена текст запроса всё равно виден", async () => {
  const { host } = mount(
    { split_suggested: true, split_request: "Раздели тему `local-runtime`" },
    { noClipboard: true },
  );
  await settled();

  host.querySelector("[data-copy]").click();
  await settled();

  assert.match(host.querySelector("[data-request]").textContent, /Раздели тему/);
  assert.match(host.textContent, new RegExp(ru.review.copyManually));
});

test("без предложения разделить блока запроса нет", async () => {
  const { host } = mount();
  await settled();

  assert.equal(host.querySelector("[data-split]"), null);
});
