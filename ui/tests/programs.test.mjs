import assert from "node:assert/strict";
import test from "node:test";

import { ru } from "../src/i18n/ru.ts";
import { settled } from "./support/dom.mjs";
import { named, refusal } from "./support/generation.mjs";
import { CHIPTUNE, NES, SHELF, programsScreen } from "./support/programs.mjs";

const { mount } = programsScreen();

test("выбор файла и перетаскивание дают один импорт", async () => {
  const picked = mount();
  await settled();
  picked.host.querySelector("[data-import]").click();
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
  host.querySelector("[data-import]").click();
  await settled();

  assert.equal(named(calls, "library").length, 2, "библиотека не перечитана");
  assert.equal(said.at(-1).tone, "ok");
  assert.match(said.at(-1).text, /Chiptune/);
});

test("копия уже импортированной программы приходит уведомлением", async () => {
  const { host, said } = mount({ imported: { uuid: "copy", title: SHELF.title, copy_of: CHIPTUNE } });
  await settled();
  host.querySelector("[data-import]").click();
  await settled();

  assert.equal(said.at(-1).tone, "info");
  assert.match(said.at(-1).text, new RegExp(ru.programs.copied));
  assert.equal(host.querySelector("[data-refused]"), null);
});

test("программа v1 получает ответ «не открывается», а не ошибку разбора", async () => {
  const { host, calls, said } = mount({ fail: refusal("package.v1", "программы v1 не открываются") });
  await settled();
  host.querySelector("[data-import]").click();
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
    host.querySelector("[data-import]").click();
    await settled();

    assert.ok(host.querySelector("[data-refused]").textContent.includes(reason), fail.code);
    dispose();
  }
});

test("пока импорт идёт, второй запуск не начинается", async () => {
  const { host, calls, drop } = mount({ hold: true });
  await settled();
  host.querySelector("[data-import]").click();
  await settled();
  drop(["/incoming/second.tolearn"]);
  host.querySelector("[data-import]").click();
  await settled();

  const button = host.querySelector("[data-import]");
  assert.equal(named(calls, "import_package").length, 1, "импорт запущен дважды");
  assert.equal(button.disabled, false, "выключенная кнопка сбросит фокус");
  assert.equal(button.getAttribute("aria-disabled"), "true");
});

test("отменённый выбор файла ничего не импортирует", async () => {
  const { host, calls } = mount({ pick: () => Promise.resolve(null) });
  await settled();
  host.querySelector("[data-import]").click();
  await settled();

  assert.equal(named(calls, "import_package").length, 0);
});

test("после удаления библиотека называет удалённую программу", async () => {
  globalThis.location = { search: `?${new URLSearchParams({ deleted: NES.title })}` };
  const { said } = mount();
  await settled();
  delete globalThis.location;

  assert.deepEqual(said.at(0), { tone: "ok", text: `${ru.programs.deleted}: ${NES.title}` });
});

test("без метки удаления библиотека молчит", async () => {
  const { said } = mount();
  await settled();

  assert.deepEqual(said, []);
});
