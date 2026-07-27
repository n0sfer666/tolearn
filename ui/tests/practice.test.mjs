import assert from "node:assert/strict";
import test, { before } from "node:test";

import { island } from "../scripts/island.mjs";
import { ru } from "../src/i18n/ru.ts";
import { browser, settled } from "./support/dom.mjs";

let Practice;
let render;
let document;

before(
  async () => {
    ({ document } = browser(
      "https://tolearn.local/ru/practice/?program=/programs/llm-agents-base&topic=local-runtime",
    ));
    ({ default: Practice } = await island("Practice"));
    ({ render } = await import("solid-js/web"));
  },
  { timeout: 300_000 },
);

const CHECK = (id) => ({ id, claim: `утверждение ${id}`, check: `команда ${id}`, expect: `ожидание ${id}` });

const OUT = {
  id: "local-runtime",
  title: "Локальный рантайм",
  practice: {
    kind: "ops",
    tier: "P2",
    task: "Подними модель локально",
    deliverable: "Лог запуска с числами",
    starting_point: null,
    fallback: "Начни с самой маленькой модели",
    time_box_min: 90,
    smoke_checked: true,
    constraints: [CHECK("c1")],
    acceptance: [CHECK("a1"), CHECK("a2")],
  },
};

function mount(options = {}) {
  const host = document.createElement("div");
  document.body.append(host);
  const calls = [];
  const call = (name, payload) => {
    calls.push({ name, payload });
    if (name === "topic") return Promise.resolve(OUT);
    if (name === "run_check") {
      if (options.refuse) return Promise.reject(new Error("проверка не запускается"));
      return Promise.resolve({
        id: payload.check,
        command: `команда ${payload.check}`,
        expect: `ожидание ${payload.check}`,
        code: options.code ?? 0,
        timed_out: options.timedOut ?? false,
        stdout: `вывод ${payload.check}`,
        stderr: "",
        truncated: false,
      });
    }
    throw new Error(`лишняя команда ${name}`);
  };
  render(
    () =>
      Practice({
        text: ru,
        locale: "ru",
        program: "/programs/llm-agents-base",
        topic: "local-runtime",
        today: "2026-07-27",
        call,
      }),
    host,
  );
  return { host, calls };
}

const item = (host, id) => host.querySelector(`[data-check="${id}"]`);

test("экран читает тему и показывает задачу до всякого запуска", async () => {
  const { host, calls } = mount();
  await settled();

  assert.deepEqual(calls.map(({ name }) => name), ["topic"]);
  assert.match(host.querySelector("[data-task]").textContent, /Подними модель локально/);
  assert.match(host.textContent, new RegExp(ru.topic.smoke));
});

test("текст команды виден до запуска, а сам запуск не случается сам", async () => {
  const { host, calls } = mount();
  await settled();

  assert.match(item(host, "c1").textContent, /команда c1/);
  assert.equal(item(host, "c1").querySelector("[data-output]"), null);
  assert.equal(calls.length, 1, "команда бандла запущена без человека");
});

test("ограничения и приёмка — две коллекции", async () => {
  const { host } = mount();
  await settled();

  const constraints = host.querySelector("[data-constraints]");
  const acceptance = host.querySelector("[data-acceptance]");
  assert.match(constraints.textContent, /команда c1/);
  assert.doesNotMatch(constraints.textContent, /команда a1/);
  assert.match(acceptance.textContent, /команда a1/);
  assert.match(acceptance.textContent, /команда a2/);
});

test("прогон идёт по нажатию и только у своей проверки", async () => {
  const { host, calls } = mount();
  await settled();

  item(host, "a1").querySelector("[data-run]").click();
  await settled();

  assert.deepEqual(calls[1], {
    name: "run_check",
    payload: { bundle: "/programs/llm-agents-base", topic: "local-runtime", check: "a1" },
  });
  assert.equal(item(host, "a2").querySelector("[data-output]"), null, "прогнана чужая проверка");

  item(host, "c1").querySelector("[data-run]").click();
  await settled();

  assert.equal(calls[2].payload.check, "c1");
  assert.match(item(host, "c1").querySelector("[data-output]").textContent, /вывод c1/);
});

test("вывод встаёт рядом с ожиданием, код возврата — подсказка", async () => {
  const { host } = mount({ code: 1 });
  await settled();

  item(host, "a1").querySelector("[data-run]").click();
  await settled();

  const done = item(host, "a1");
  assert.match(done.querySelector("[data-output]").textContent, /вывод a1/);
  assert.match(done.querySelector("[data-expect]").textContent, /ожидание a1/);
  assert.match(done.querySelector("[data-code]").textContent, /1/);
});

test("истёкшее время названо словом, а не пустым выводом", async () => {
  const { host } = mount({ timedOut: true, code: null });
  await settled();

  item(host, "a1").querySelector("[data-run]").click();
  await settled();

  assert.match(item(host, "a1").textContent, new RegExp(ru.practice.timedOut));
});

test("незапустившаяся проверка говорит об этом и не роняет экран", async () => {
  const { host } = mount({ refuse: true });
  await settled();

  item(host, "a1").querySelector("[data-run]").click();
  await settled();

  assert.match(item(host, "a1").textContent, new RegExp(ru.practice.failed));
  assert.ok(host.querySelector("[data-task]"), "экран уцелел");
});

test("отметку о выполнении ставит человек, а не код возврата", async () => {
  const { host } = mount({ code: 0 });
  await settled();

  const check = item(host, "a1");
  assert.equal(check.querySelector("[data-done]").getAttribute("aria-pressed"), "false");

  check.querySelector("[data-run]").click();
  await settled();
  assert.equal(
    item(host, "a1").querySelector("[data-done]").getAttribute("aria-pressed"),
    "false",
    "нулевой код возврата поставил отметку сам",
  );

  item(host, "a1").querySelector("[data-done]").click();
  await settled();
  assert.equal(item(host, "a1").querySelector("[data-done]").getAttribute("aria-pressed"), "true");
});

test("с практики есть ход обратно на тему", async () => {
  const { host } = mount();
  await settled();

  const back = host.querySelector("[data-topic-link]");
  assert.match(back.getAttribute("href"), /\/ru\/topic\/\?program=/);
  assert.match(back.getAttribute("href"), /topic=local-runtime/);
});
