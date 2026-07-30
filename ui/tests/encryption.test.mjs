import assert from "node:assert/strict";
import test, { before } from "node:test";

import { island } from "../scripts/island.mjs";
import { ru } from "../src/i18n/ru.ts";
import { browser, settled, toasts } from "./support/dom.mjs";

let Encryption;
let render;
let document;

before(
  async () => {
    ({ document } = browser("https://tolearn.local/ru/settings/"));
    ({ default: Encryption } = await island("Encryption"));
    ({ render } = await import("solid-js/web"));
  },
  { timeout: 300_000 },
);

function mount(options = {}) {
  const host = document.createElement("div");
  document.body.append(host);
  const calls = [];
  const state = { enabled: options.enabled === true, external: options.external === true };
  const call = (name, payload) => {
    calls.push({ name, payload });
    if (name !== "encryption") throw new Error(`лишняя команда ${name}`);
    if (payload.enable === null) return Promise.resolve({ ...state });
    if (options.refuse === true) {
      return Promise.reject(new Error("не получилось"));
    }
    state.enabled = payload.enable;
    return Promise.resolve({ ...state });
  };
  const said = toasts(document.defaultView);
  render(() => Encryption({ text: ru, call }), host);
  return { host, calls, said };
}

test("до включения человек видит предупреждение о потере конспектов", async () => {
  const { host } = mount();
  await settled();

  assert.equal(host.querySelector("[data-loss]").textContent, ru.encryption.loss);
  assert.equal(host.querySelector("[data-enable]").disabled, true);
});

test("шифрование включается только с фразой и подтверждённым предупреждением", async () => {
  const { host, calls } = mount();
  await settled();

  const phrase = host.querySelector("[data-phrase]");
  phrase.value = "длинная парольная фраза";
  phrase.dispatchEvent(new document.defaultView.Event("input", { bubbles: true }));
  await settled();
  assert.equal(host.querySelector("[data-enable]").disabled, true, "без подтверждения включает");

  const warned = host.querySelector("[data-warned]");
  warned.checked = true;
  warned.dispatchEvent(new document.defaultView.Event("change", { bubbles: true }));
  await settled();

  host.querySelector("[data-enable]").click();
  await settled();

  assert.deepEqual(calls[1], {
    name: "encryption",
    payload: { enable: true, phrase: "длинная парольная фраза" },
  });
  assert.equal(host.querySelector("[data-enabled]").textContent, ru.encryption.enabled);
});

test("при включённом шифровании предлагается расшифровка", async () => {
  const { host, calls } = mount({ enabled: true });
  await settled();

  assert.equal(host.querySelector("[data-phrase]"), null, "фраза спрашивается зря");
  host.querySelector("[data-disable]").click();
  await settled();

  assert.deepEqual(calls[1].payload, { enable: false, phrase: "" });
  assert.equal(host.querySelector("[data-loss]").textContent, ru.encryption.loss);
});

test("внешний каталог закрывает включение и говорит почему", async () => {
  const { host } = mount({ external: true });
  await settled();

  assert.equal(host.querySelector("[data-external]").textContent, ru.encryption.external);
  assert.equal(host.querySelector("[data-enable]"), null, "кнопка включения осталась");
});

test("отказ ядра показывается человеку", async () => {
  const { host, said } = mount({ refuse: true });
  await settled();

  const phrase = host.querySelector("[data-phrase]");
  phrase.value = "длинная парольная фраза";
  phrase.dispatchEvent(new document.defaultView.Event("input", { bubbles: true }));
  const warned = host.querySelector("[data-warned]");
  warned.checked = true;
  warned.dispatchEvent(new document.defaultView.Event("change", { bubbles: true }));
  await settled();

  host.querySelector("[data-enable]").click();
  await settled();

  assert.deepEqual(said.at(-1), { tone: "error", text: "не получилось" });
});
