import assert from "node:assert/strict";
import test, { before } from "node:test";

import { island } from "../scripts/island.mjs";
import { ru } from "../src/i18n/ru.ts";
import { browser, settled, toasts } from "./support/dom.mjs";

let Notes;
let render;
let document;

before(
  async () => {
    ({ document } = browser(
      "https://tolearn.local/ru/notes/?program=/programs/llm-agents-base&topic=local-runtime",
    ));
    ({ default: Notes } = await island("Notes"));
    ({ render } = await import("solid-js/web"));
  },
  { timeout: 300_000 },
);

const STAMP = (size) => ({ modified_nanos: `${size}000`, size });

function mount(options = {}) {
  const host = document.createElement("div");
  document.body.append(host);
  const calls = [];
  let reads = 0;
  const call = (name, payload) => {
    calls.push({ name, payload });
    if (name === "note") {
      reads += 1;
      const body = reads === 1 ? (options.body ?? "") : (options.reread ?? options.body ?? "");
      return Promise.resolve({
        body,
        path: body === "" ? null : "/данные/notes/llm-agents-base/local-runtime.md",
        stamp: body === "" ? null : STAMP(reads),
      });
    }
    if (name === "save_note") {
      if (options.conflict === true) {
        return Promise.resolve({ saved: false, stamp: null, theirs: "Версия снаружи" });
      }
      return Promise.resolve({ saved: true, stamp: STAMP(9), theirs: null });
    }
    throw new Error(`лишняя команда ${name}`);
  };
  const said = toasts(document.defaultView);
  render(
    () =>
      Notes({
        text: ru,
        locale: "ru",
        program: "/programs/llm-agents-base",
        topic: "local-runtime",
        call,
      }),
    host,
  );
  return { host, calls, said };
}

const area = (host) => host.querySelector("[data-note]");

test("конспект читается при открытии экрана", async () => {
  const { host, calls } = mount({ body: "Старый текст" });
  await settled();

  assert.deepEqual(calls, [
    {
      name: "note",
      payload: { bundle: "/programs/llm-agents-base", topic: "local-runtime", directory: null },
    },
  ]);
  assert.equal(area(host).value, "Старый текст");
});

test("пустой конспект открывается пустым полем без ошибки", async () => {
  const { host } = mount({ body: "" });
  await settled();

  assert.equal(area(host).value, "");
  assert.equal(host.querySelector("[data-conflict]"), null);
});

test("запись уходит вместе со слепком прочитанного", async () => {
  const { host, calls, said } = mount({ body: "Старый текст" });
  await settled();

  area(host).value = "Новый текст";
  area(host).dispatchEvent(new document.defaultView.Event("input", { bubbles: true }));
  host.querySelector("[data-save]").click();
  await settled();

  assert.deepEqual(calls[1], {
    name: "save_note",
    payload: {
      bundle: "/programs/llm-agents-base",
      topic: "local-runtime",
      body: "Новый текст",
      directory: null,
      stamp: STAMP(1),
    },
  });
  assert.deepEqual(said.at(-1), { tone: "ok", text: ru.notes.saved });
});

test("одновременная правка показывает обе версии и ничего не сливает", async () => {
  const { host, said } = mount({ body: "Старый текст", conflict: true });
  await settled();

  area(host).value = "Моя версия";
  area(host).dispatchEvent(new document.defaultView.Event("input", { bubbles: true }));
  host.querySelector("[data-save]").click();
  await settled();

  const conflict = host.querySelector("[data-conflict]");
  assert.match(conflict.querySelector("[data-theirs]").textContent, /Версия снаружи/);
  assert.match(conflict.querySelector("[data-ours]").textContent, /Моя версия/);
  assert.equal(area(host).value, "Моя версия", "поле затёрто чужой версией");
  assert.deepEqual(said.at(-1), { tone: "warn", text: ru.notes.conflict });
});

test("правка снаружи подхватывается по перечитыванию", async () => {
  const { host, calls } = mount({ body: "Старый текст", reread: "Правка снаружи" });
  await settled();

  host.querySelector("[data-reread]").click();
  await settled();

  assert.equal(calls[1].name, "note");
  assert.equal(area(host).value, "Правка снаружи");
});

test("с конспекта есть ход обратно на тему", async () => {
  const { host } = mount({ body: "" });
  await settled();

  const back = host.querySelector("[data-topic-link]");
  assert.match(back.getAttribute("href"), /\/ru\/topic\/\?program=/);
  assert.match(back.getAttribute("href"), /topic=local-runtime/);
});
