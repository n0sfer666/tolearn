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
  journal: false,
  local: {
    endpoint: "http://127.0.0.1:11434",
    api: "ollama",
    model: "",
    num_ctx: 0,
    temperature_tenths: 7,
  },
  remote: { endpoint: "", api: "openai", model: "", num_ctx: 0, temperature_tenths: 7 },
  harness: { id: "claude", command: "claude", args: ["-p"], timeout_secs: 180 },
};

const ADVISED = [
  {
    id: "qwen3:4b",
    repo: "unsloth/Qwen3-4B-GGUF:Q4_K_M",
    gigabytes: 3,
    heavy: false,
    installed: true,
  },
  {
    id: "qwen3:32b",
    repo: "unsloth/Qwen3-32B-GGUF:Q4_K_M",
    gigabytes: 20,
    heavy: true,
    installed: false,
  },
];

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
  const logs = [];
  const logged = { room: "/данные/llm-log", records: options.records ?? 0 };
  const call = (name, payload) => {
    if (name === "llm_log") {
      logs.push(payload);
      if (payload.clear === true) logged.records = 0;
      return Promise.resolve({ ...logged });
    }
    calls.push({ name, payload });
    if (name !== "provider") throw new Error(`лишняя команда ${name}`);
    if (payload.save === null && payload.forget === false) {
      return Promise.resolve({
        provider: { ...DEFAULTS, ...options.stored },
        has_key: options.hasKey === true,
        checked: null,
        probed: null,
        presets: PRESETS,
        advised: ADVISED,
      });
    }
    if (options.refuse !== undefined) {
      return Promise.reject({ code: options.refuse, message: options.refusal ?? "" });
    }
    return Promise.resolve({
      provider: payload.save ?? { ...DEFAULTS, ...options.stored },
      has_key: payload.forget === true ? false : payload.key !== null || options.hasKey === true,
      checked: payload.check === true ? (options.checked ?? { models: ["llama3:8b"], version: null, took_ms: null }) : null,
      probed:
        payload.probe === true
          ? { said: options.thinking ?? "готов", took_ms: 3680, thinking: options.thinking != null }
          : null,
      presets: PRESETS,
      advised: ADVISED,
    });
  };
  const said = toasts(document.defaultView);
  render(() => Provider({ text: ru, locale: "ru", call }), host);
  return { host, calls, logs, said };
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
    local: { ...DEFAULTS.local, endpoint: "http://127.0.0.1:1234" },
  });
  assert.equal(calls[1].payload.check, false);
  assert.deepEqual(said.at(-1), { tone: "ok", text: ru.provider.saved });
});

test("настройки неактивных видов уходят вместе с активным", async () => {
  const { host, calls } = mount({
    stored: { local: { ...DEFAULTS.local, endpoint: "http://здесь", model: "qwen3" } },
  });
  await settled();

  host.querySelector("[data-kind=harness]").click();
  host.querySelector("[data-save]").click();
  await settled();

  assert.equal(calls[1].payload.save.active, "harness");
  assert.deepEqual(calls[1].payload.save.local, {
    ...DEFAULTS.local,
    endpoint: "http://здесь",
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
    ...DEFAULTS.local,
    endpoint: "http://127.0.0.1:8080/v1",
    api: "openai",
  });
});

test("свой адрес смена API не затирает", async () => {
  const { host, calls } = mount({
    stored: {
      local: { ...DEFAULTS.local, endpoint: "http://192.168.1.10:9000/v1", model: "qwen3" },
    },
  });
  await settled();

  host.querySelector("[data-api=openai]").click();
  host.querySelector("[data-save]").click();
  await settled();

  assert.equal(calls[1].payload.save.local.endpoint, "http://192.168.1.10:9000/v1");
  assert.equal(calls[1].payload.save.local.api, "openai");
});

test("контекст спрашивают только у ollama", async () => {
  const { host } = mount();
  await settled();

  assert.notEqual(host.querySelector("[data-num-ctx]"), null);
  assert.equal(host.querySelector("[data-context-hint]"), null);

  host.querySelector("[data-api=openai]").click();
  await settled();

  assert.equal(host.querySelector("[data-num-ctx]"), null);
  assert.equal(host.querySelector("[data-context-hint]").textContent, ru.provider.contextHint);
});

test("контекст и температура уходят в сохранение числами", async () => {
  const { host, calls } = mount();
  await settled();

  input(host, "[data-num-ctx]", "16384");
  input(host, "[data-temperature]", "0.3");
  host.querySelector("[data-save]").click();
  await settled();

  assert.equal(calls[1].payload.save.local.num_ctx, 16384);
  assert.equal(calls[1].payload.save.local.temperature_tenths, 3);
});

test("у внешнего сервиса выбора API нет", async () => {
  const { host } = mount({ stored: { active: "remote" } });
  await settled();

  assert.equal(host.querySelector("[data-api=ollama]"), null);
  assert.equal(host.querySelector("[data-key]").value, "");
});

test("пресет харнесса подставляет команду, а поле аргументов оставляет пустым", async () => {
  const { host, calls } = mount({ stored: { active: "harness" } });
  await settled();

  const preset = host.querySelector("[data-preset]");
  preset.value = "opencode";
  preset.dispatchEvent(new document.defaultView.Event("change", { bubbles: true }));
  await settled();

  assert.equal(host.querySelector("[data-command]").value, "opencode");
  assert.equal(host.querySelector("[data-args]").value, "");

  host.querySelector("[data-save]").click();
  await settled();
  assert.deepEqual(calls[1].payload.save.harness.args, []);
});

test("рекомендованные аргументы подставляются кнопкой, а не сами", async () => {
  const { host, calls } = mount({
    stored: { active: "harness", harness: { ...DEFAULTS.harness, args: [] } },
  });
  await settled();

  host.querySelector("[data-args-advise]").click();
  await settled();

  assert.equal(host.querySelector("[data-args]").value, "-p");
  host.querySelector("[data-save]").click();
  await settled();
  assert.deepEqual(calls[1].payload.save.harness.args, ["-p"]);
});

test("своей команде подставлять нечего — кнопка выключена", async () => {
  const { host } = mount({ stored: { active: "harness", harness: { ...DEFAULTS.harness, id: "custom" } } });
  await settled();

  assert.equal(host.querySelector("[data-args-advise]").disabled, true);
});

test("аргументы правятся построчно", async () => {
  const { host, calls } = mount({ stored: { active: "harness" } });
  await settled();

  input(host, "[data-args]", "-p\n--allowedTools\n");
  host.querySelector("[data-save]").click();
  await settled();

  assert.deepEqual(calls[1].payload.save.harness.args, ["-p", "--allowedTools", ""]);
});

test("пустой аргумент виден в перечне того, что уйдёт харнессу", async () => {
  const { host } = mount({ stored: { active: "harness" } });
  await settled();

  input(host, "[data-args]", "-p\n--allowedTools\n");
  await settled();
  const shown = host.querySelector("[data-args-seen]").textContent;
  assert.match(shown, /3/);
  assert.match(shown, /\(пусто\)/);

  input(host, "[data-args]", "-p\n--allowedTools");
  await settled();
  const short = host.querySelector("[data-args-seen]").textContent;
  assert.match(short, /2/);
  assert.doesNotMatch(short, /\(пусто\)/);
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

test("проверка харнесса называет версию и время ответа", async () => {
  const { host } = mount({
    stored: { enabled: true, active: "harness" },
    checked: { models: [], version: "2.1.220", took_ms: 5300 },
  });
  await settled();

  host.querySelector("[data-check]").click();
  await settled();

  const said = host.querySelector("[data-checked]").textContent;
  assert.match(said, /2\.1\.220/);
  assert.match(said, /5\.3/);
  assert.match(said, new RegExp(ru.provider.took));
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

test("текст отказа харнесса виден на экране, а не только в тосте", async () => {
  const complaint = "харнесс завершился с кодом 1: error: option '--allowedTools' argument missing";
  const { host } = mount({
    stored: { enabled: true, active: "harness" },
    refuse: "harness.failed",
    refusal: complaint,
  });
  await settled();

  host.querySelector("[data-check]").click();
  await settled();

  assert.equal(host.querySelector("[data-refusal]").textContent, complaint);

  host.querySelector("[data-check]").click();
  assert.equal(host.querySelector("[data-refusal]"), null, "прошлый отказ остался висеть");
});

test("не-харнессный отказ на экран дословно не выносится", async () => {
  const { host } = mount({
    stored: { enabled: true, active: "local" },
    refuse: "provider.unreachable",
    refusal: "connection refused",
  });
  await settled();

  host.querySelector("[data-check]").click();
  await settled();

  assert.equal(host.querySelector("[data-refusal]"), null);
});

test("совет по моделям называет размер, годность и команду установки", async () => {
  const { host } = mount();
  await settled();

  const rows = host.querySelectorAll("[data-advice]");
  assert.equal(rows.length, ADVISED.length);
  assert.equal(rows[0].querySelector("[data-advice-model]").textContent, "qwen3:4b");
  assert.match(rows[0].querySelector("[data-advice-size]").textContent, /3 ГБ/);
  assert.equal(rows[0].querySelector("[data-advice-fit]").textContent, ru.provider.modelInstalled);
  assert.equal(rows[0].querySelector("[data-advice-command]").textContent, "ollama pull qwen3:4b");
  assert.equal(rows[1].querySelector("[data-advice-fit]").textContent, ru.provider.modelHeavy);
});

test("смена API переписывает совет, не дожидаясь сохранения", async () => {
  const { host } = mount();
  await settled();

  host.querySelector("[data-api=openai]").click();
  await settled();

  const first = host.querySelectorAll("[data-advice]")[0];
  assert.equal(
    first.querySelector("[data-advice-model]").textContent,
    "unsloth/Qwen3-4B-GGUF:Q4_K_M",
  );
  assert.equal(
    first.querySelector("[data-advice-command]").textContent,
    "llama-server -hf unsloth/Qwen3-4B-GGUF:Q4_K_M",
  );
});

test("совет по моделям не показывают внешнему сервису", async () => {
  const { host } = mount({ stored: { active: "remote" } });
  await settled();

  assert.equal(host.querySelector("[data-advice]"), null);
});

test("выбор модели из совета попадает в поле", async () => {
  const { host } = mount();
  await settled();

  host.querySelectorAll("[data-advice]")[1].querySelector("[data-advice-pick]").click();
  await settled();

  assert.equal(host.querySelector("[data-model]").value, "qwen3:32b");
});

test("отсутствие модели на сервере объясняется словами", async () => {
  const { host, said } = mount({ stored: { enabled: true }, refuse: "provider.model-missing" });
  await settled();

  host.querySelector("[data-check]").click();
  await settled();

  assert.deepEqual(said.at(-1), { tone: "error", text: ru.provider.modelMissing });
});

test("неизвестный код отказа не оставляет экран без объяснения", async () => {
  const { host, said } = mount({ stored: { enabled: true }, refuse: "provider.невиданный" });
  await settled();

  host.querySelector("[data-save]").click();
  await settled();

  assert.deepEqual(said.at(-1), { tone: "error", text: ru.provider.failed });
});

test("журнал выключен по умолчанию и показывает счётчик с путём", async () => {
  const { host, logs } = mount({ records: 3 });
  await settled();

  assert.equal(host.querySelector("[data-journal]").checked, false);
  assert.deepEqual(logs, [{ open: false, clear: false }]);
  assert.equal(
    host.querySelector("[data-journal-room]").textContent,
    `${ru.provider.journalKept} 3 · /данные/llm-log`,
  );
});

test("галочка журнала уходит в сохранение провайдера", async () => {
  const { host, calls } = mount();
  await settled();

  host.querySelector("[data-journal]").click();
  await settled();
  host.querySelector("[data-save]").click();
  await settled();

  assert.equal(calls.at(-1).payload.save.journal, true);
});

test("очистка журнала обнуляет счётчик", async () => {
  const { host, logs } = mount({ records: 7 });
  await settled();

  host.querySelector("[data-journal-clear]").click();
  await settled();

  assert.deepEqual(logs.at(-1), { open: false, clear: true });
  assert.equal(
    host.querySelector("[data-journal-room]").textContent,
    `${ru.provider.journalKept} 0 · /данные/llm-log`,
  );
});

test("открытие папки журнала не трогает записи", async () => {
  const { host, logs } = mount({ records: 2 });
  await settled();

  host.querySelector("[data-journal-open]").click();
  await settled();

  assert.deepEqual(logs.at(-1), { open: true, clear: false });
  assert.equal(
    host.querySelector("[data-journal-room]").textContent,
    `${ru.provider.journalKept} 2 · /данные/llm-log`,
  );
});
