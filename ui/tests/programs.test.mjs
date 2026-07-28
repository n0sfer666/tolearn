import assert from "node:assert/strict";
import test, { before } from "node:test";

import { island } from "../scripts/island.mjs";
import { ru } from "../src/i18n/ru.ts";
import { browser, settled } from "./support/dom.mjs";

let Programs;
let render;
let document;

before(async () => {
  ({ document } = browser());
  ({ default: Programs } = await island("Programs"));
  ({ render } = await import("solid-js/web"));
}, { timeout: 300_000 });

const CARD = {
  id: "llm-agents-base",
  title: "Работа с агентами LLM",
  path: "/programs/llm-agents-base",
  reachable: true,
  opened_at: "2026-07-27",
  tally: { done: 3, total: 8, stale: 1, share: 0.375, hours_done: { min: 4, max: 6 }, hours_total: { min: 12, max: 20 } },
};

const MERGED = {
  ok: true,
  id: CARD.id,
  title: CARD.title,
  violations: [],
  report: { kept: ["a", "b"], added: ["c"], orphaned: ["d"], stale: [{ id: "a", changed: ["outcomes"] }] },
};

function mount(options = {}) {
  const host = document.createElement("div");
  document.body.append(host);
  const calls = [];
  const listing = options.listing ?? [CARD];
  let drop = () => {};
  const call = (name, payload) => {
    calls.push({ name, payload });
    if (name === "programs") return Promise.resolve({ programs: listing });
    if (name === "import") return options.hold ? new Promise(() => {}) : Promise.resolve(options.imported ?? MERGED);
    throw new Error(`лишняя команда ${name}`);
  };
  const dispose = render(
    () =>
      Programs({
        text: ru.programs,
        call,
        pick: options.pick ?? (() => Promise.resolve("/dropped/bundle")),
        pickArchive: options.pickArchive ?? (() => Promise.resolve("/dropped/bundle.zip")),
        drops: (handler) => {
          drop = handler;
        },
      }),
    host,
  );
  return { host, calls, dispose, drop: (paths) => drop(paths) };
}

test("список показывает программу и её прогресс", async () => {
  const { host } = mount();
  await settled();

  assert.match(host.textContent, /Работа с агентами LLM/);
  assert.match(host.textContent, /3\D+8/, `прогресс не виден: ${host.textContent}`);
});

test("недоступная программа помечена, а не пропала", async () => {
  const { host } = mount({ listing: [{ ...CARD, reachable: false, tally: null }] });
  await settled();

  assert.match(host.textContent, /Работа с агентами LLM/, "запись исчезла");
  assert.match(host.textContent, new RegExp(ru.programs.unreachable));
});

test("выбор папки и перетаскивание дают один результат", async () => {
  const picked = mount();
  await settled();
  picked.host.querySelector("[data-pick]").click();
  await settled();

  const dropped = mount();
  await settled();
  dropped.drop(["/dropped/bundle"]);
  await settled();

  const imports = ({ calls }) => calls.filter(({ name }) => name === "import");
  assert.deepEqual(imports(picked), imports(dropped), "пути импорта расходятся");
  assert.equal(imports(picked).length, 1);
  assert.equal(imports(picked)[0].payload.path, "/dropped/bundle");
  assert.equal(picked.host.textContent, dropped.host.textContent, "экраны после импорта разные");
});

test("битая папка объясняет причину и не показывает отчёт", async () => {
  const refused = {
    ok: false,
    id: null,
    title: null,
    violations: [{ code: "bundle.missing-topic-file", message: "нет файла темы `local-runtime`" }],
    report: null,
  };
  const { host } = mount({ imported: refused });
  await settled();
  host.querySelector("[data-pick]").click();
  await settled();

  assert.match(host.textContent, /нет файла темы/, host.textContent);
  assert.doesNotMatch(host.textContent, new RegExp(ru.programs.added));
});

test("повторный импорт показывает отчёт о слиянии", async () => {
  const { host } = mount();
  await settled();
  host.querySelector("[data-pick]").click();
  await settled();

  const report = host.querySelector("[data-report]").textContent;
  assert.match(report, new RegExp(`${ru.programs.added}\\D+1`), report);
  assert.match(report, new RegExp(`${ru.programs.stale}\\D+1`), report);
  assert.match(report, new RegExp(`${ru.programs.orphaned}\\D+1`), report);
  assert.match(report, /outcomes/, "не сказано, что изменилось");
});

test("список перечитывается после удачного импорта, но не после отказа", async () => {
  const good = mount();
  await settled();
  good.host.querySelector("[data-pick]").click();
  await settled();
  assert.equal(good.calls.filter(({ name }) => name === "programs").length, 2, "список не обновлён");

  const bad = mount({ imported: { ok: false, id: null, title: null, violations: [{ code: "x", message: "нет" }], report: null } });
  await settled();
  bad.host.querySelector("[data-pick]").click();
  await settled();
  assert.equal(bad.calls.filter(({ name }) => name === "programs").length, 1, "отказ зря дёрнул список");
});

test("пока импорт идёт, второй запуск не начинается", async () => {
  const { host, calls, drop } = mount({ hold: true });
  await settled();
  host.querySelector("[data-pick]").click();
  await settled();
  drop(["/second/bundle"]);
  host.querySelector("[data-pick]").click();
  await settled();

  assert.equal(calls.filter(({ name }) => name === "import").length, 1, "импорт запущен дважды");
});

test("отменённый выбор папки ничего не импортирует", async () => {
  const { host, calls } = mount({ pick: () => Promise.resolve(null) });
  await settled();
  host.querySelector("[data-pick]").click();
  await settled();

  assert.equal(calls.filter(({ name }) => name === "import").length, 0);
});

test("фильтр оставляет в списке только совпавшие программы", async () => {
  const other = { ...CARD, id: "rust-core", title: "Ядро на Rust", path: "/programs/rust-core" };
  const { host } = mount({ listing: [CARD, other] });
  await settled();

  const field = host.querySelector("[data-filter]");
  field.value = "ядро";
  field.dispatchEvent(new Event("input", { bubbles: true }));
  await settled();

  assert.equal(host.querySelector('[data-program="llm-agents-base"]'), null);
  assert.ok(host.querySelector('[data-program="rust-core"]'), "совпавшая программа пропала");
});

test("архив выбирается своей кнопкой и уходит в тот же импорт", async () => {
  const { host, calls } = mount();
  await settled();

  host.querySelector("[data-pick-archive]").click();
  await settled();

  const imports = calls.filter(({ name }) => name === "import");
  assert.equal(imports.length, 1);
  assert.equal(imports[0].payload.path, "/dropped/bundle.zip");
});

test("отменённый выбор архива импорт не запускает", async () => {
  const { host, calls } = mount({ pickArchive: () => Promise.resolve(null) });
  await settled();

  host.querySelector("[data-pick-archive]").click();
  await settled();

  assert.equal(calls.filter(({ name }) => name === "import").length, 0);
});
