import assert from "node:assert/strict";
import test, { after, before } from "node:test";

import { island } from "../scripts/island.mjs";
import { ru } from "../src/i18n/ru.ts";
import { browser, settled } from "./support/dom.mjs";

let Generate;
let render;
let document;

before(async () => {
  ({ document } = browser());
  ({ default: Generate } = await island("Generate"));
  ({ render } = await import("solid-js/web"));
}, { timeout: 300_000 });

const LIVE = {
  step: "skeleton",
  total: 0,
  done: 0,
  attempt: 1,
  rounds: 3,
  current: "",
  waiting: false,
  finished: false,
  cancelled: false,
  refused: [],
  seconds: 4,
  step_seconds: 2,
  chars: 0,
  ticks: 0,
  tail: "",
  tokens: 320,
  summary: null,
};

const SUMMARY = {
  id: "rust-core",
  title: "Ядро на Rust",
  topics: 8,
  hours_min: 12,
  hours_max: 20,
  stages: [{ n: 1, title: "Основы владения", topics: 3, first: ["Владение", "Заимствование"] }],
};

const CARD = {
  id: "rust-core",
  title: "Ядро на Rust",
  path: "/data/unpacked/rust-core",
  reachable: true,
  opened_at: "2026-08-06",
  tally: null,
};

const IMPORTED = { ok: true, id: "rust-core", title: "Ядро на Rust", violations: [], report: null };

const alive = [];

after(() => {
  for (const dispose of alive) dispose();
});

function mount(options = {}) {
  const host = document.createElement("div");
  document.body.append(host);
  const calls = [];
  const opened = [];
  const states = options.states ?? [LIVE];
  const call = (name, payload) => {
    calls.push({ name, payload });
    if (name === "provider") return Promise.resolve({ provider: { enabled: options.enabled ?? true } });
    if (name === "generate") return Promise.resolve({ job: "gen-1" });
    if (name === "generate_state") return Promise.resolve(states.length > 1 ? states.shift() : states[0]);
    if (name === "generate_go") return Promise.resolve({ going: true });
    if (name === "generate_stop") return Promise.resolve({ stopping: true });
    if (name === "generate_accept") return Promise.resolve(options.imported ?? IMPORTED);
    if (name === "programs") return Promise.resolve({ programs: [CARD] });
    throw new Error(`лишняя команда ${name}`);
  };
  const dispose = render(
    () =>
      Generate({
        text: ru.generate,
        locale: options.locale ?? "ru",
        today: "2026-08-06",
        step: 1,
        call,
        open: (href) => opened.push(href),
      }),
    host,
  );
  alive.push(dispose);
  return { host, calls, opened, dispose };
}

const ticks = async (times = 4) => {
  for (let i = 0; i < times; i += 1) await new Promise((resolve) => setTimeout(resolve, 4));
};

async function ask(host, subject = "хочу разобраться в устройстве Rust") {
  host.querySelector("[data-generate-open]").click();
  await settled();
  const field = host.querySelector("[data-subject]");
  field.value = subject;
  field.dispatchEvent(new Event("input", { bubbles: true }));
  await settled();
  host.querySelector("[data-ask]").dispatchEvent(new Event("submit", { bubbles: true, cancelable: true }));
  await ticks();
}

test("выключенный провайдер гасит кнопку и говорит, чем она включается", async () => {
  const { host, calls } = mount({ enabled: false });
  await settled();

  assert.equal(host.querySelector("[data-generate-open]").disabled, true);
  assert.match(host.textContent, new RegExp(ru.generate.off));
  assert.equal(calls.filter(({ name }) => name === "generate").length, 0);
});

test("одна фраза человека и уровень уходят в генерацию", async () => {
  const { host, calls } = mount();
  await settled();
  await ask(host);

  const [started] = calls.filter(({ name }) => name === "generate");
  assert.equal(started.payload.subject, "хочу разобраться в устройстве Rust");
  assert.equal(started.payload.level, "basics");
  assert.equal(started.payload.weekly_hours, 6);
  assert.equal(started.payload.weeks, 10);
});

test("после скелета экран показывает объём и ждёт подтверждения", async () => {
  const waiting = { ...LIVE, step: "confirm", total: 8, waiting: true };
  const { host, calls } = mount({ states: [waiting] });
  await settled();
  await ask(host);

  assert.match(host.textContent, new RegExp(`${ru.generate.estimate}: 8`), host.textContent);
  host.querySelector("[data-generate-go]").click();
  await settled();

  assert.equal(calls.filter(({ name }) => name === "generate_go").length, 1);
});

test("прогресс называет тему, круг починки и цену запроса", async () => {
  const live = { ...LIVE, step: "topic", total: 8, done: 2, attempt: 2, current: "Владение", tokens: null };
  const { host } = mount({ states: [live] });
  await settled();
  await ask(host);

  const seen = host.querySelector("[data-generating]").textContent;
  assert.match(seen, /Тема 3 \/ 8: Владение/, seen);
  assert.match(seen, new RegExp(`${ru.generate.attempt} 2 \\/ 3`), seen);
  assert.match(seen, new RegExp(ru.generate.silent), seen);
});

test("пока модель печатает, окно называет объём, время шага и хвост ответа", async () => {
  const live = {
    ...LIVE,
    step: "topic",
    total: 8,
    done: 2,
    current: "Владение",
    seconds: 269,
    step_seconds: 72,
    chars: 2413,
    ticks: 9,
    tail: "первая строка\nвторая строка\nтретья строка\nчетвёртая строка\nпятая строка",
  };
  const { host } = mount({ states: [live] });
  await settled();
  await ask(host);

  const beat = host.querySelector("[data-generate-beat]").textContent;
  assert.match(beat, new RegExp(`2413 ${ru.generate.letters}`), beat);
  assert.match(beat, new RegExp(`${ru.generate.here} 1 ${ru.generate.minutes} 12 ${ru.generate.seconds}`), beat);
  assert.match(beat, new RegExp(`${ru.generate.whole} 4 ${ru.generate.minutes} 29 ${ru.generate.seconds}`), beat);
  assert.equal(beat.startsWith(`✻ ${ru.generate.crunch[3]}`), true, beat);

  const tail = host.querySelector("[data-generate-tail]").textContent;
  assert.equal(tail.includes("первая строка"), false, tail);
  assert.equal(tail.includes("пятая строка"), true, tail);
});

test("сводка говорит, сколько заняла сборка", async () => {
  const done = { ...LIVE, step: "done", finished: true, seconds: 1790, summary: SUMMARY };
  const { host } = mount({ states: [done] });
  await settled();
  await ask(host);

  const beat = host.querySelector("[data-generate-beat]").textContent;
  assert.match(beat, new RegExp(`${ru.generate.built} 29 ${ru.generate.minutes} 50 ${ru.generate.seconds}`), beat);
  assert.equal(host.querySelector("[data-generate-tail]"), null);
});

test("сводка показывается до записи, «Принять» уводит на новую программу", async () => {
  const done = { ...LIVE, step: "done", total: 8, done: 8, finished: true, summary: SUMMARY };
  const { host, calls, opened } = mount({ states: [done] });
  await settled();
  await ask(host);

  const summary = host.querySelector("[data-generate-summary]").textContent;
  assert.match(summary, /Ядро на Rust/, summary);
  assert.match(summary, /12–20/, summary);
  assert.match(summary, /Основы владения/, summary);
  assert.equal(calls.filter(({ name }) => name === "generate_accept").length, 0, "записали без согласия");

  host.querySelector("[data-generate-accept]").click();
  await ticks();

  assert.equal(calls.filter(({ name }) => name === "generate_accept").length, 1);
  assert.deepEqual(opened, ["/ru/program/?program=%2Fdata%2Funpacked%2Frust-core"]);
});

test("отказ приёмки показан словами и на программу не уводит", async () => {
  const done = { ...LIVE, step: "done", total: 8, done: 8, finished: true, summary: SUMMARY };
  const refused = {
    ok: false,
    id: null,
    title: null,
    violations: [{ code: "bundle.missing-topic-file", message: "нет файла темы `ownership`" }],
    report: null,
  };
  const { host, opened } = mount({ states: [done], imported: refused });
  await settled();
  await ask(host);
  host.querySelector("[data-generate-accept]").click();
  await ticks();

  assert.match(host.textContent, /нет файла темы/, host.textContent);
  assert.deepEqual(opened, []);
});

test("отмена на сводке останавливает генерацию и ничего не принимает", async () => {
  const done = { ...LIVE, step: "done", total: 8, done: 8, finished: true, summary: SUMMARY };
  const { host, calls } = mount({ states: [done] });
  await settled();
  await ask(host);
  host.querySelector("[data-generate-close]").click();
  await ticks();

  assert.equal(calls.filter(({ name }) => name === "generate_stop").length, 1);
  assert.equal(calls.filter(({ name }) => name === "generate_accept").length, 0);
  assert.equal(host.querySelector("[data-generate-summary]"), null, "сводка осталась на экране");
});

test("три круга брака показаны нарушениями, а принимать нечего", async () => {
  const stuck = {
    ...LIVE,
    step: "skeleton",
    attempt: 3,
    finished: true,
    refused: ["bundle.unknown-topic: тема `x` вне этапов"],
  };
  const { host } = mount({ states: [stuck] });
  await settled();
  await ask(host);

  assert.match(host.textContent, new RegExp(ru.generate.stopped), host.textContent);
  assert.match(host.textContent, /bundle\.unknown-topic/, host.textContent);
  assert.equal(host.querySelector("[data-generate-accept]"), null, "предлагают принять брак");
});
