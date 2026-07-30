import assert from "node:assert/strict";
import test, { before } from "node:test";

import { island } from "../scripts/island.mjs";
import { ru } from "../src/i18n/ru.ts";
import { browser, settled } from "./support/dom.mjs";

let Settings;
let render;
let document;

before(
  async () => {
    ({ document } = browser("https://tolearn.local/ru/settings/"));
    ({ default: Settings } = await island("Settings"));
    ({ render } = await import("solid-js/web"));
  },
  { timeout: 300_000 },
);

const DEFAULTS = {
  disk_budget_mb: 2048,
  notes_directory: null,
  locale: "ru",
  theme: "system",
  history_depth: 5,
  history_share_percent: 10,
};

function mount(options = {}) {
  const host = document.createElement("div");
  document.body.append(host);
  const calls = [];
  const call = (name, payload) => {
    calls.push({ name, payload });
    if (name !== "settings") throw new Error(`лишняя команда ${name}`);
    if (payload.save === null) return Promise.resolve({ ...DEFAULTS, ...options.stored });
    if (options.refuse === true) {
      return Promise.reject({ code: "settings.unknown-value", message: "нет" });
    }
    return Promise.resolve(payload.save);
  };
  render(
    () =>
      Settings({
        text: ru,
        locale: "ru",
        call,
        choose: () => Promise.resolve(options.chosen ?? null),
      }),
    host,
  );
  return { host, calls };
}

test("настройки читаются при открытии экрана", async () => {
  const { host, calls } = mount({ stored: { disk_budget_mb: 512, theme: "dark" } });
  await settled();

  assert.deepEqual(calls[0], { name: "settings", payload: { save: null } });
  assert.equal(host.querySelector("[data-budget]").value, "512");
  assert.equal(host.querySelector("[data-notes]").textContent, ru.settings.notesOwn);
  assert.equal(
    host.querySelector('[data-theme-choice="dark"]').getAttribute("aria-pressed"),
    "true",
  );
});

test("новый бюджет диска уходит в ядро сразу", async () => {
  const { host, calls } = mount();
  await settled();

  const input = host.querySelector("[data-budget]");
  input.value = "128";
  input.dispatchEvent(new window.Event("change", { bubbles: true }));
  await settled();

  assert.equal(calls.at(-1).payload.save.disk_budget_mb, 128);
  assert.equal(host.querySelector("[data-saved]").textContent, ru.settings.saved);
});

test("нулевой бюджет не уезжает в ядро", async () => {
  const { host, calls } = mount();
  await settled();

  const input = host.querySelector("[data-budget]");
  input.value = "0";
  input.dispatchEvent(new window.Event("change", { bubbles: true }));
  await settled();

  assert.equal(calls.length, 1, "запрос ушёл с нулевым бюджетом");
});

test("глубина истории и её доля бюджета уходят в ядро", async () => {
  const { host, calls } = mount({ stored: { history_depth: 3, history_share_percent: 25 } });
  await settled();

  assert.equal(host.querySelector("[data-history-depth]").value, "3");
  assert.equal(host.querySelector("[data-history-share]").value, "25");

  const depth = host.querySelector("[data-history-depth]");
  depth.value = "2";
  depth.dispatchEvent(new window.Event("change", { bubbles: true }));
  await settled();
  assert.equal(calls.at(-1).payload.save.history_depth, 2);

  const share = host.querySelector("[data-history-share]");
  share.value = "40";
  share.dispatchEvent(new window.Event("change", { bubbles: true }));
  await settled();
  assert.equal(calls.at(-1).payload.save.history_share_percent, 40);
});

test("доля бюджета больше ста процентов в ядро не уезжает", async () => {
  const { host, calls } = mount();
  await settled();

  const share = host.querySelector("[data-history-share]");
  share.value = "140";
  share.dispatchEvent(new window.Event("change", { bubbles: true }));
  await settled();

  assert.equal(calls.length, 1, "запрос ушёл с долей больше ста процентов");
});

test("выбранный каталог конспектов сохраняется и виден", async () => {
  const { host, calls } = mount({ chosen: "/данные/конспекты" });
  await settled();

  host.querySelector("[data-choose]").click();
  await settled();

  assert.equal(calls.at(-1).payload.save.notes_directory, "/данные/конспекты");
  assert.equal(host.querySelector("[data-notes]").textContent, "/данные/конспекты");
});

test("отказ от внешнего каталога возвращает свой", async () => {
  const { host, calls } = mount({ stored: { notes_directory: "/данные/конспекты" } });
  await settled();

  host.querySelector("[data-reset]").click();
  await settled();

  assert.equal(calls.at(-1).payload.save.notes_directory, null);
  assert.equal(host.querySelector("[data-notes]").textContent, ru.settings.notesOwn);
  assert.equal(host.querySelector("[data-reset]"), null);
});

test("переключатель темы предлагает три варианта на языке экрана", async () => {
  const { host } = mount();
  await settled();

  const choices = [...host.querySelectorAll("[data-theme-choice]")];
  assert.deepEqual(
    choices.map((button) => button.dataset.themeChoice),
    ["system", "light", "dark"],
  );
  assert.deepEqual(
    choices.map((button) => button.textContent),
    [ru.theme.system, ru.theme.light, ru.theme.dark],
  );
});

test("тема применяется к странице без перезагрузки", async () => {
  const { host } = mount();
  await settled();

  host.querySelector('[data-theme-choice="dark"]').click();
  await settled();

  assert.equal(document.documentElement.dataset.theme, "dark");
  assert.equal(window.localStorage.getItem("tolearn.theme"), "dark");

  host.querySelector('[data-theme-choice="system"]').click();
  await settled();

  assert.equal(document.documentElement.dataset.theme, undefined);
  assert.equal(window.localStorage.getItem("tolearn.theme"), null);
});

test("язык — ссылка на тот же экран и запись в настройки", async () => {
  const { host, calls } = mount();
  await settled();

  const link = host.querySelector('[data-locale="en"]');
  assert.equal(link.getAttribute("href"), "/en/settings/");
  link.click();
  await settled();

  assert.equal(calls.at(-1).payload.save.locale, "en");
});

test("отказ ядра виден на экране", async () => {
  const { host } = mount({ refuse: true });
  await settled();

  host.querySelector('[data-theme-choice="light"]').click();
  await settled();

  assert.equal(host.querySelector("[data-failed]").textContent, ru.settings.failed);
  assert.equal(host.querySelector("[data-saved]"), null);
});
