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
  active: "local",
  local: { endpoint: "http://127.0.0.1:11434", api: "ollama", model: "" },
  remote: { endpoint: "", api: "openai", model: "" },
  harness: { id: "claude", command: "claude", args: ["-p"], timeout_secs: 180 },
};

const PRESETS = [
  { id: "claude", command: "claude", args: ["-p"] },
  { id: "opencode", command: "opencode", args: ["run"] },
  { id: "pi", command: "pi", args: ["-p", "--no-tools"] },
  { id: "custom", command: "", args: [] },
];

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
        probed: null,
        presets: PRESETS,
      });
    }
    if (options.refuse !== undefined) return Promise.reject({ code: options.refuse });
    return Promise.resolve({
      provider: payload.save ?? { ...DEFAULTS, ...options.stored },
      has_key: payload.forget === true ? false : payload.key !== null || options.hasKey === true,
      checked: payload.check === true ? { models: ["llama3:8b"], version: null } : null,
      probed:
        payload.probe === true
          ? { said: options.thinking ?? "готов", took_ms: 3680, thinking: options.thinking != null }
          : null,
      presets: PRESETS,
    });
  };
  const said = toasts(document.defaultView);
  render(() => Provider({ text: ru, locale: "ru", call }), host);
  return { host, calls, said };
}

function input(host, selector, value) {
  const field = host.querySelector(selector);
  field.value = value;
  field.dispatchEvent(new document.defaultView.Event("input", { bubbles: true }));
}

test("провайдер читается при открытии экрана и выключен по умолчанию", async () => {
  const { host, calls } = mount();
  await settled();

  assert.deepEqual(calls[0], {
    name: "provider",
    payload: { save: null, key: null, forget: false, check: false, probe: false },
  });
  assert.equal(host.querySelector("[data-enabled]").checked, false);
  assert.equal(host.querySelector("[data-kind=local]").checked, true);
  assert.equal(host.querySelector("[data-endpoint]").value, DEFAULTS.local.endpoint);
  assert.equal(host.querySelector("[data-privacy]").textContent, ru.provider.localPrivacy);
});

test("у локальной модели не спрашивают ключ", async () => {
  const { host } = mount();
  await settled();

  assert.equal(host.querySelector("[data-key]"), null);
  assert.equal(host.querySelector("[data-command]"), null);
});

test("смена вида показывает свои поля и предупреждает про внешний сервис", async () => {
  const { host } = mount();
  await settled();

  host.querySelector("[data-kind=harness]").click();
  await settled();

  assert.equal(host.querySelector("[data-command]").value, "claude");
  assert.equal(host.querySelector("[data-endpoint]"), null);
  assert.equal(host.querySelector("[data-privacy]").textContent, ru.provider.outsidePrivacy);
  assert.equal(host.querySelector("[data-args-warning]").textContent, ru.provider.argsWarning);
});

test("правки уходят на сохранение одной командой", async () => {
  const { host, calls, said } = mount();
  await settled();

  host.querySelector("[data-enabled]").click();
  input(host, "[data-endpoint]", "http://127.0.0.1:1234");
  host.querySelector("[data-save]").click();
  await settled();

  assert.deepEqual(calls[1].payload.save, {
    ...DEFAULTS,
    enabled: true,
    local: { endpoint: "http://127.0.0.1:1234", api: "ollama", model: "" },
  });
  assert.equal(calls[1].payload.check, false);
  assert.deepEqual(said.at(-1), { tone: "ok", text: ru.provider.saved });
});

test("настройки неактивных видов уходят вместе с активным", async () => {
  const { host, calls } = mount({
    stored: { local: { endpoint: "http://здесь", api: "ollama", model: "qwen3" } },
  });
  await settled();

  host.querySelector("[data-kind=harness]").click();
  host.querySelector("[data-save]").click();
  await settled();

  assert.equal(calls[1].payload.save.active, "harness");
  assert.deepEqual(calls[1].payload.save.local, {
    endpoint: "http://здесь",
    api: "ollama",
    model: "qwen3",
  });
});

test("локальной модели выбирают API, и типовой адрес подставляется сам", async () => {
  const { host, calls } = mount();
  await settled();

  assert.equal(host.querySelector("[data-api=ollama]").checked, true);
  host.querySelector("[data-api=openai]").click();
  host.querySelector("[data-save]").click();
  await settled();

  assert.equal(host.querySelector("[data-endpoint]").value, "http://127.0.0.1:8080/v1");
  assert.deepEqual(calls[1].payload.save.local, {
    endpoint: "http://127.0.0.1:8080/v1",
    api: "openai",
    model: "",
  });
});

test("свой адрес смена API не затирает", async () => {
  const { host, calls } = mount({
    stored: { local: { endpoint: "http://192.168.1.10:9000/v1", api: "ollama", model: "qwen3" } },
  });
  await settled();

  host.querySelector("[data-api=openai]").click();
  host.querySelector("[data-save]").click();
  await settled();

  assert.equal(calls[1].payload.save.local.endpoint, "http://192.168.1.10:9000/v1");
  assert.equal(calls[1].payload.save.local.api, "openai");
});

test("у внешнего сервиса выбора API нет", async () => {
  const { host } = mount({ stored: { active: "remote" } });
  await settled();

  assert.equal(host.querySelector("[data-api=ollama]"), null);
  assert.equal(host.querySelector("[data-key]").value, "");
});

test("пресет харнесса подставляет команду и аргументы", async () => {
  const { host, calls } = mount({ stored: { active: "harness" } });
  await settled();

  const preset = host.querySelector("[data-preset]");
  preset.value = "opencode";
  preset.dispatchEvent(new document.defaultView.Event("change", { bubbles: true }));
  host.querySelector("[data-save]").click();
  await settled();

  assert.equal(host.querySelector("[data-command]").value, "opencode");
  assert.deepEqual(calls[1].payload.save.harness.args, ["run"]);
});

test("аргументы правятся построчно", async () => {
  const { host, calls } = mount({ stored: { active: "harness" } });
  await settled();

  input(host, "[data-args]", "-p\n--allowedTools\n");
  host.querySelector("[data-save]").click();
  await settled();

  assert.deepEqual(calls[1].payload.save.harness.args, ["-p", "--allowedTools", ""]);
});

test("ключ уходит отдельным полем и не остаётся в форме", async () => {
  const { host, calls } = mount({ stored: { active: "remote" } });
  await settled();

  input(host, "[data-key]", "sk-секрет");
  host.querySelector("[data-save]").click();
  await settled();

  assert.equal(calls[1].payload.key, "sk-секрет");
  assert.equal(JSON.stringify(calls[1].payload.save).includes("sk-секрет"), false);
  assert.equal(host.querySelector("[data-key]").value, "");
  assert.equal(host.querySelector("[data-stored]").textContent, ru.provider.keyStored);
});

test("сохранённый ключ можно забыть", async () => {
  const { host, calls } = mount({ stored: { active: "remote" }, hasKey: true });
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
  assert.equal(calls[1].payload.probe, false);
  assert.match(host.querySelector("[data-checked]").textContent, /llama3:8b/);
});

test("пробный запрос показывает ответ модели", async () => {
  const { host, calls } = mount({ stored: { enabled: true } });
  await settled();

  host.querySelector("[data-probe]").click();
  await settled();

  assert.equal(calls[1].payload.probe, true);
  assert.match(host.querySelector("[data-probed]").textContent, /готов/);
});

test("размышление вместо ответа не выдаётся за ответ", async () => {
  const { host } = mount({ stored: { enabled: true }, thinking: "Thinking Process: 1. Analyze" });
  await settled();

  host.querySelector("[data-probe]").click();
  await settled();

  const shown = host.querySelector("[data-probed]").textContent;
  assert.match(shown, /3\.7/);
  assert.doesNotMatch(shown, /Thinking Process/);
});

test("отказ провайдера объясняется словами", async () => {
  const { host, said } = mount({ stored: { enabled: true }, refuse: "provider.rejected" });
  await settled();

  host.querySelector("[data-check]").click();
  await settled();

  assert.deepEqual(said.at(-1), { tone: "error", text: ru.provider.rejected });
  assert.equal(host.querySelector("[data-checked]"), null);
});

test("отказ харнесса объясняется своими словами", async () => {
  const { host, said } = mount({
    stored: { enabled: true, active: "harness" },
    refuse: "harness.not-found",
  });
  await settled();

  host.querySelector("[data-check]").click();
  await settled();

  assert.deepEqual(said.at(-1), { tone: "error", text: ru.provider.notFound });
});

test("неизвестный код отказа не оставляет экран без объяснения", async () => {
  const { host, said } = mount({ stored: { enabled: true }, refuse: "provider.невиданный" });
  await settled();

  host.querySelector("[data-save]").click();
  await settled();

  assert.deepEqual(said.at(-1), { tone: "error", text: ru.provider.failed });
});
