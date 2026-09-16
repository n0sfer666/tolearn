import assert from "node:assert/strict";
import test, { before } from "node:test";

import { island } from "../scripts/island.mjs";
import { ru } from "../src/i18n/ru.ts";
import { browser, settled, toasts } from "./support/dom.mjs";

let Ledger;
let render;
let document;

before(
  async () => {
    ({ document } = browser("https://tolearn.local/ru/settings/"));
    ({ default: Ledger } = await island("Ledger"));
    ({ render } = await import("solid-js/web"));
  },
  { timeout: 300_000 },
);

function mount(options = {}) {
  const host = document.createElement("div");
  document.body.append(host);
  const logs = [];
  const logged = { room: "/данные/llm-log", records: options.records ?? 0 };
  const call = (name, payload) => {
    if (name !== "llm_log") throw new Error(`лишняя команда ${name}`);
    logs.push(payload);
    if (options.refuse === true) return Promise.reject({ code: "llm-log.unreadable", message: "нет" });
    if (payload.clear === true) logged.records = 0;
    return Promise.resolve({ ...logged });
  };
  const said = toasts(document.defaultView);
  render(() => Ledger({ text: ru, call }), host);
  return { host, logs, said };
}

const room = (host) => host.querySelector("[data-journal-room]").textContent;

test("журнал читается при открытии и показывает счётчик с путём", async () => {
  const { host, logs } = mount({ records: 3 });
  await settled();

  assert.deepEqual(logs, [{ open: false, clear: false }]);
  assert.equal(host.querySelector('[data-card="journal"] h2').textContent, ru.journal.title);
  assert.equal(room(host), `${ru.journal.kept} 3 · /данные/llm-log`);
});

test("очистка журнала обнуляет счётчик", async () => {
  const { host, logs } = mount({ records: 7 });
  await settled();

  host.querySelector("[data-journal-clear]").click();
  await settled();

  assert.deepEqual(logs.at(-1), { open: false, clear: true });
  assert.equal(room(host), `${ru.journal.kept} 0 · /данные/llm-log`);
});

test("открытие папки журнала не трогает записи", async () => {
  const { host, logs } = mount({ records: 2 });
  await settled();

  host.querySelector("[data-journal-open]").click();
  await settled();

  assert.deepEqual(logs.at(-1), { open: true, clear: false });
  assert.equal(room(host), `${ru.journal.kept} 2 · /данные/llm-log`);
});

test("отказ журнала сказан, а не проглочен", async () => {
  const { host, said } = mount({ refuse: true });
  await settled();

  assert.equal(host.querySelector("[data-journal-room]"), null);
  assert.equal(said.at(-1).tone, "error");
});
