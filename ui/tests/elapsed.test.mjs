import assert from "node:assert/strict";
import test, { after } from "node:test";

import { ru } from "../src/i18n/ru.ts";
import { clocked } from "../src/lib/clocked.ts";
import { elapsed } from "../src/lib/elapsed.ts";

const { createRoot, createSignal } = await import("solid-js");

const alive = [];

after(() => {
  for (const dispose of alive) dispose();
});

const waited = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

function counter(now) {
  const [since, setSince] = createSignal(null);
  const spent = createRoot((dispose) => {
    alive.push(dispose);
    return elapsed(since, 10, now);
  });
  return { spent, setSince };
}

test("без запущенной работы счётчик стоит на нуле", () => {
  const { spent } = counter(() => 5_000);

  assert.equal(spent(), 0);
});

test("счётчик встаёт на прошедшее сразу, не дожидаясь первого такта", () => {
  let clock = 10_000;
  const { spent, setSince } = counter(() => clock);

  setSince(7_500);

  assert.equal(spent(), 2_500);
});

test("счётчик идёт дальше сам, такт за тактом", async () => {
  let clock = 10_000;
  const { spent, setSince } = counter(() => clock);

  setSince(10_000);
  clock = 11_000;
  await waited(30);

  assert.equal(spent(), 1_000);
});

test("конец работы возвращает счётчик на нуль и гасит такт", async () => {
  let clock = 10_000;
  const { spent, setSince } = counter(() => clock);
  setSince(10_000);
  await waited(30);

  setSince(null);
  clock = 30_000;
  await waited(30);

  assert.equal(spent(), 0);
});

test("время до минуты читается секундами, дальше — минутами и секундами", () => {
  const cases = [
    [0, "0 с"],
    [999, "0 с"],
    [45_400, "45 с"],
    [59_900, "59 с"],
    [60_000, "1:00"],
    [80_000, "1:20"],
    [605_000, "10:05"],
  ];

  for (const [ms, said] of cases) assert.equal(clocked(ms, ru), said, `${ms} мс`);
});
