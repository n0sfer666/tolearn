import assert from "node:assert/strict";
import test from "node:test";

import { ru } from "../src/i18n/ru.ts";
import { settled } from "./support/dom.mjs";
import { OUT, stageScreen } from "./support/stage.mjs";

const { mount } = stageScreen();

test("практика читается целиком, запуск стоит только у пункта с командой", async () => {
  const { host, calls } = mount();
  await settled();

  const practice = host.querySelector("[data-practice]");
  for (const seen of ["Сыграйте гамму", "Файл melody.nsf", "Только пульсы", "Гамма звучит", "python play.py", "exit 0"]) {
    assert.ok(practice.textContent.includes(seen), seen);
  }
  assert.deepEqual(
    [...host.querySelectorAll("[data-question]")].map((node) => node.id),
    ["q1", "q2"],
  );
  assert.equal(host.querySelector("#c1 [data-run]"), null, "у пункта без команды есть кнопка");
  assert.ok(host.querySelector("#a1 [data-run]"));
  assert.equal(
    host.querySelectorAll("button:not([data-snip], [data-regenerate], [data-run], [data-choose], [data-exam])").length,
    0,
    "появилась лишняя кнопка",
  );
  assert.deepEqual(
    calls.map((made) => made.name),
    ["stage"],
    "команда ушла без нажатия",
  );
});

test("галочка стоит по состоянию и уходит в него, ничего не запуская", async () => {
  const { host, calls } = mount({
    out: { ...OUT, ticks: ["c1"] },
    replies: { tick: (payload) => ({ ticks: payload.on ? ["c1", payload.claim] : [] }) },
  });
  await settled();
  const box = (id) => host.querySelector(`#${id} input[data-tick]`);
  assert.equal(box("c1").checked, true);
  assert.equal(box("a1").checked, false);

  box("a1").click();
  await settled();

  assert.deepEqual(calls.at(-1), {
    name: "tick",
    payload: { program: "chip", node: "chip", stage: "voices", claim: "a1", on: true },
  });
  assert.equal(box("a1").checked, true);
  assert.equal(host.querySelector("#a1 [data-output]"), null, "галочка запустила проверку");
});

test("незаписанная галочка возвращается как была", async () => {
  const { host } = mount({
    replies: {
      tick: () => {
        throw { code: "state.unwritable", message: "нет места" };
      },
    },
  });
  await settled();
  const box = host.querySelector("#a1 input[data-tick]");

  box.click();
  await settled();

  assert.equal(box.checked, false);
});

test("без папки кнопка просит её выбрать и ничего не запускает", async () => {
  const { host, calls, picks } = mount({
    folder: "/tmp/практика",
    replies: { workdir: (payload) => ({ workdir: payload.path }) },
  });
  await settled();
  const run = host.querySelector("#a1 [data-run]");
  assert.equal(run.textContent, ru.stage.choose);
  assert.match(host.querySelector("[data-workdir]").textContent, new RegExp(ru.stage.unset));

  run.click();
  await settled();

  assert.equal(picks.length, 1);
  assert.deepEqual(
    calls.map((made) => made.name),
    ["stage", "workdir"],
  );
  assert.deepEqual(calls[1].payload, { program: "chip", path: "/tmp/практика" });
  assert.equal(host.querySelector("[data-workdir] [data-path]").textContent, "/tmp/практика");
  assert.equal(run.textContent, ru.stage.run);
});

test("отменённый выбор папки ничего не пишет", async () => {
  const { host, calls, picks } = mount();
  await settled();

  host.querySelector("[data-choose]").click();
  await settled();

  assert.equal(picks.length, 1);
  assert.deepEqual(
    calls.map((made) => made.name),
    ["stage"],
  );
  assert.equal(host.querySelector("[data-workdir] [data-path]"), null);
});

test("проверка идёт по нажатию, и вывод с кодом стоят рядом с ожиданием без вердикта", async () => {
  const { host, calls } = mount({
    out: { ...OUT, workdir: "/w" },
    replies: {
      check_claim: () => ({ outcome: "finished", code: 3, stdout: "said\n", stderr: "moaned\n", truncated: false }),
    },
  });
  await settled();
  assert.equal(host.querySelector("[data-workdir] [data-path]").textContent, "/w");
  assert.equal(host.querySelector("[data-choose]").textContent, ru.stage.change);

  host.querySelector("#a1 [data-run]").click();
  await settled();

  assert.deepEqual(calls[1], {
    name: "check_claim",
    payload: { program: "chip", node: "chip", stage: "voices", claim: "a1" },
  });
  const claim = host.querySelector("#a1");
  assert.ok(claim.querySelector("[data-expect]"));
  assert.equal(claim.querySelector("[data-stdout]").textContent, "said\n");
  assert.equal(claim.querySelector("[data-stderr]").textContent, "moaned\n");
  assert.equal(claim.querySelector("[data-code]").textContent, ru.stage.code.replace("{code}", "3"));
  assert.equal(claim.querySelector("[data-limit]"), null);
  assert.equal(claim.querySelector("input[data-tick]").checked, false, "приложение вынесло вердикт");
});

test("сработавший лимит и обрезанный вывод видны", async () => {
  const { host } = mount({
    out: { ...OUT, workdir: "/w" },
    replies: {
      check_claim: () => ({ outcome: "timeout", code: null, stdout: "x", stderr: "", truncated: true }),
    },
  });
  await settled();

  host.querySelector("#a1 [data-run]").click();
  await settled();

  const output = host.querySelector("#a1 [data-output]");
  assert.equal(output.querySelector("[data-limit]").textContent, ru.stage.timeout);
  assert.equal(output.querySelector("[data-truncated]").textContent, ru.stage.truncated);
  assert.equal(output.querySelector("[data-code]"), null);
  assert.equal(output.querySelector("[data-stderr]"), null);
});

test("исчезнувшая папка называет причину у пункта", async () => {
  const { host } = mount({
    out: { ...OUT, workdir: "/w" },
    replies: {
      check_claim: () => {
        throw { code: "workdir.absent", message: "папки `/w` нет" };
      },
    },
  });
  await settled();

  host.querySelector("#a1 [data-run]").click();
  await settled();

  assert.equal(host.querySelector("#a1 [data-failed]").textContent, ru.stage.gone);
  assert.equal(host.querySelector("#a1 [data-output]"), null);
});
