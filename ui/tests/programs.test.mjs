import assert from "node:assert/strict";
import test, { before } from "node:test";

import { island } from "../scripts/island.mjs";
import { ru } from "../src/i18n/ru.ts";
import { browser, settled, toasts } from "./support/dom.mjs";

let Programs;
let render;
let window;

before(async () => {
  window = browser();
  ({ default: Programs } = await island("Programs"));
  ({ render } = await import("solid-js/web"));
}, { timeout: 300_000 });

const CHIPTUNE = "0d4f6c8a-2b1e-4f3a-9c5d-7e8f9a0b1c2d";
const span = (min, max) => ({ min, max });

const SHELF = {
  uuid: CHIPTUNE,
  title: "Chiptune: музыка звукового чипа NES",
  goal: "Написать и проиграть мелодию на пяти голосах",
  hours: span(7, 11),
  summary: { passed: 0, total: 1, skipped: 0 },
  active: null,
  unread: null,
};

const NES = {
  uuid: "nes-dev",
  title: "Разработка игр для NES",
  goal: "Собрать игру",
  hours: span(14, 22),
  summary: { passed: 1, total: 2, skipped: 0 },
  active: "2026-09-10",
  unread: null,
};

const refusal = (code, message) => ({ code, message });

function mount(options = {}) {
  const host = window.document.createElement("div");
  window.document.body.append(host);
  const calls = [];
  let drop = () => {};
  const call = (name, payload) => {
    calls.push({ name, payload });
    if (name === "library") {
      return Promise.resolve({ programs: options.listing ?? [SHELF], refused: options.refused ?? [] });
    }
    if (name !== "import_package") throw new Error(`лишняя команда ${name}`);
    if (options.hold) return new Promise(() => {});
    if (options.fail) return Promise.reject(options.fail);
    return Promise.resolve(options.imported ?? { uuid: CHIPTUNE, title: SHELF.title, copy_of: null });
  };
  const said = toasts(window);
  const dispose = render(
    () =>
      Programs({
        text: ru.programs,
        locale: options.locale ?? "ru",
        call,
        pick: options.pick ?? (() => Promise.resolve("/incoming/chiptune.tolearn")),
        drops: (handler) => {
          drop = handler;
        },
      }),
    host,
  );
  return { host, calls, said, dispose, drop: (paths) => drop(paths) };
}

const named = (calls, name) => calls.filter((made) => made.name === name);

test("карточка несёт название, цель и часы карты", async () => {
  const { host } = mount();
  await settled();

  const card = host.querySelector(`[data-program="${CHIPTUNE}"]`);
  assert.match(card.textContent, /Chiptune: музыка звукового чипа NES/);
  assert.match(card.querySelector("[data-goal]").textContent, /Написать и проиграть/);
  assert.match(card.querySelector("[data-hours]").textContent, /7\D+11/);
});

test("битая запись библиотеки видна с причиной, а не пропала", async () => {
  const { host } = mount({
    refused: [{ directory: "5f0e", code: "library.malformed", message: "program.yaml не читается" }],
  });
  await settled();

  const broken = host.querySelector("[data-broken]");
  assert.match(broken.textContent, /5f0e/);
  assert.match(broken.textContent, /program\.yaml не читается/);
  assert.match(broken.textContent, new RegExp(ru.programs.broken));
});

test("выбор файла и перетаскивание дают один импорт", async () => {
  const picked = mount();
  await settled();
  picked.host.querySelector("[data-pick]").click();
  await settled();

  const dropped = mount();
  await settled();
  dropped.drop(["/incoming/chiptune.tolearn"]);
  await settled();

  assert.deepEqual(named(picked.calls, "import_package"), named(dropped.calls, "import_package"));
  assert.deepEqual(named(picked.calls, "import_package"), [
    { name: "import_package", payload: { path: "/incoming/chiptune.tolearn" } },
  ]);
});

test("удачный импорт называет программу и перечитывает библиотеку", async () => {
  const { host, calls, said } = mount();
  await settled();
  host.querySelector("[data-pick]").click();
  await settled();

  assert.equal(named(calls, "library").length, 2, "библиотека не перечитана");
  assert.equal(said.at(-1).tone, "ok");
  assert.match(said.at(-1).text, /Chiptune/);
});

test("копия уже импортированной программы приходит уведомлением", async () => {
  const { host, said } = mount({ imported: { uuid: "copy", title: SHELF.title, copy_of: CHIPTUNE } });
  await settled();
  host.querySelector("[data-pick]").click();
  await settled();

  assert.equal(said.at(-1).tone, "info");
  assert.match(said.at(-1).text, new RegExp(ru.programs.copied));
  assert.equal(host.querySelector("[data-refused]"), null);
});

test("программа v1 получает ответ «не открывается», а не ошибку разбора", async () => {
  const { host, calls, said } = mount({ fail: refusal("package.v1", "программы v1 не открываются") });
  await settled();
  host.querySelector("[data-pick]").click();
  await settled();

  const refused = host.querySelector("[data-refused]");
  assert.equal(refused.getAttribute("role"), "alert");
  assert.match(refused.textContent, new RegExp(ru.programs.v1));
  assert.deepEqual(said.at(-1), { tone: "error", text: ru.programs.refused });
  assert.equal(named(calls, "library").length, 1, "отказ зря перечитал библиотеку");
});

test("папка программы и чужой файл получают свои причины", async () => {
  const cases = [
    [refusal("package.folder", "это папка"), ru.programs.folder],
    [refusal("package.foreign", "не пакет"), ru.programs.foreign],
    [refusal("archive.malformed", "архив битый на 12-м байте"), "архив битый на 12-м байте"],
  ];
  for (const [fail, reason] of cases) {
    const { host, dispose } = mount({ fail });
    await settled();
    host.querySelector("[data-pick]").click();
    await settled();

    assert.ok(host.querySelector("[data-refused]").textContent.includes(reason), fail.code);
    dispose();
  }
});

test("пока импорт идёт, второй запуск не начинается", async () => {
  const { host, calls, drop } = mount({ hold: true });
  await settled();
  host.querySelector("[data-pick]").click();
  await settled();
  drop(["/incoming/second.tolearn"]);
  host.querySelector("[data-pick]").click();
  await settled();

  const button = host.querySelector("[data-pick]");
  assert.equal(named(calls, "import_package").length, 1, "импорт запущен дважды");
  assert.equal(button.disabled, false, "выключенная кнопка сбросит фокус");
  assert.equal(button.getAttribute("aria-disabled"), "true");
});

test("отменённый выбор файла ничего не импортирует", async () => {
  const { host, calls } = mount({ pick: () => Promise.resolve(null) });
  await settled();
  host.querySelector("[data-pick]").click();
  await settled();

  assert.equal(named(calls, "import_package").length, 0);
});

test("фильтр оставляет только совпавшие программы", async () => {
  const { host } = mount({ listing: [SHELF, NES] });
  await settled();

  const field = host.querySelector("[data-filter]");
  field.value = "nes";
  field.dispatchEvent(new Event("input", { bubbles: true }));
  await settled();

  assert.ok(host.querySelector('[data-program="nes-dev"]'), "совпавшая программа пропала");
  assert.ok(host.querySelector(`[data-program="${CHIPTUNE}"]`), "совпадение по NES в названии потеряно");
  field.value = "игр";
  field.dispatchEvent(new Event("input", { bubbles: true }));
  await settled();
  assert.equal(host.querySelector(`[data-program="${CHIPTUNE}"]`), null);
});

test("фильтр прячет и битые записи, которые не совпали", async () => {
  const { host } = mount({
    refused: [{ directory: "5f0e", code: "library.malformed", message: "program.yaml не читается" }],
  });
  await settled();

  const field = host.querySelector("[data-filter]");
  field.value = "chiptune";
  field.dispatchEvent(new Event("input", { bubbles: true }));
  await settled();
  assert.equal(host.querySelector("[data-broken]"), null, "битая запись пережила фильтр");

  field.value = "5f0e";
  field.dispatchEvent(new Event("input", { bubbles: true }));
  await settled();
  assert.ok(host.querySelector("[data-broken]"), "совпавшая битая запись пропала");
});

test("ссылка на программу ведёт на её экран текущего языка", async () => {
  const { host } = mount({ locale: "en" });
  await settled();

  assert.equal(host.querySelector("li[data-program] a").getAttribute("href"), `/en/program/?program=${CHIPTUNE}`);
});
