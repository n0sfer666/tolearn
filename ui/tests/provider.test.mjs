import assert from "node:assert/strict";
import test, { before } from "node:test";

import { island } from "../scripts/island.mjs";
import { ru } from "../src/i18n/ru.ts";
import { browser, settled, toasts } from "./support/dom.mjs";

let Provider;
let render;
let document;

before(
  async () => {
    ({ document } = browser("https://tolearn.local/ru/settings/"));
    ({ default: Provider } = await island("Provider"));
    ({ render } = await import("solid-js/web"));
  },
  { timeout: 300_000 },
);

const DEFAULTS = {
  enabled: false,
  flavor: "ollama",
  endpoint: "http://127.0.0.1:11434",
  model: "",
};

function mount(options = {}) {
  const host = document.createElement("div");
  document.body.append(host);
  const calls = [];
  const call = (name, payload) => {
    calls.push({ name, payload });
    if (name !== "provider") throw new Error(`лишняя команда ${name}`);
    if (payload.save === null && payload.forget === false) {
      return Promise.resolve({
        provider: { ...DEFAULTS, ...options.stored },
        has_key: options.hasKey === true,
        checked: null,
      });
    }
    if (options.refuse !== undefined) return Promise.reject({ code: options.refuse });
    return Promise.resolve({
      provider: payload.save ?? { ...DEFAULTS, ...options.stored },
      has_key: payload.forget === true ? false : payload.key !== null || options.hasKey === true,
      checked: payload.check === true ? { models: ["llama3:8b"] } : null,
    });
  };
  const said = toasts(document.defaultView);
  render(() => Provider({ text: ru, locale: "ru", call }), host);
  return { host, calls, said };
}

test("провайдер читается при открытии экрана и выключен по умолчанию", async () => {
  const { host, calls } = mount();
  await settled();

  assert.deepEqual(calls[0], {
    name: "provider",
    payload: { save: null, key: null, forget: false, check: false },
  });
  assert.equal(host.querySelector("[data-enabled]").checked, false);
  assert.equal(host.querySelector("[data-endpoint]").value, DEFAULTS.endpoint);
  assert.equal(host.querySelector("[data-stored]").textContent, ru.provider.keyEmpty);
  assert.equal(host.querySelector("[data-forget]"), null);
});

test("правки уходят на сохранение одной командой", async () => {
  const { host, calls, said } = mount();
  await settled();

  host.querySelector("[data-enabled]").click();
  const endpoint = host.querySelector("[data-endpoint]");
  endpoint.value = "http://127.0.0.1:1234";
  endpoint.dispatchEvent(new document.defaultView.Event("input", { bubbles: true }));
  host.querySelector("[data-save]").click();
  await settled();

  assert.deepEqual(calls[1].payload.save, {
    ...DEFAULTS,
    enabled: true,
    endpoint: "http://127.0.0.1:1234",
  });
  assert.equal(calls[1].payload.check, false);
  assert.deepEqual(said.at(-1), { tone: "ok", text: ru.provider.saved });
});

test("ключ уходит отдельным полем и не остаётся в форме", async () => {
  const { host, calls } = mount();
  await settled();

  const key = host.querySelector("[data-key]");
  key.value = "sk-секрет";
  key.dispatchEvent(new document.defaultView.Event("input", { bubbles: true }));
  host.querySelector("[data-save]").click();
  await settled();

  assert.equal(calls[1].payload.key, "sk-секрет");
  assert.equal(JSON.stringify(calls[1].payload.save).includes("sk-секрет"), false);
  assert.equal(host.querySelector("[data-key]").value, "");
  assert.equal(host.querySelector("[data-stored]").textContent, ru.provider.keyStored);
});

test("сохранённый ключ можно забыть", async () => {
  const { host, calls } = mount({ hasKey: true });
  await settled();

  host.querySelector("[data-forget]").click();
  await settled();

  assert.equal(calls[1].payload.forget, true);
  assert.equal(host.querySelector("[data-stored]").textContent, ru.provider.keyEmpty);
  assert.equal(host.querySelector("[data-forget]"), null);
});

test("проверка соединения показывает модели", async () => {
  const { host, calls } = mount({ stored: { enabled: true } });
  await settled();

  host.querySelector("[data-check]").click();
  await settled();

  assert.equal(calls[1].payload.check, true);
  assert.match(host.querySelector("[data-checked]").textContent, /llama3:8b/);
});

test("отказ провайдера объясняется словами", async () => {
  const { host, said } = mount({ stored: { enabled: true }, refuse: "provider.rejected" });
  await settled();

  host.querySelector("[data-check]").click();
  await settled();

  assert.deepEqual(said.at(-1), { tone: "error", text: ru.provider.rejected });
  assert.equal(host.querySelector("[data-checked]"), null);
});

test("неизвестный код отказа не оставляет экран без объяснения", async () => {
  const { host, said } = mount({ stored: { enabled: true }, refuse: "provider.невиданный" });
  await settled();

  host.querySelector("[data-save]").click();
  await settled();

  assert.deepEqual(said.at(-1), { tone: "error", text: ru.provider.failed });
});
