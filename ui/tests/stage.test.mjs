import assert from "node:assert/strict";
import test, { before } from "node:test";

import { island } from "../scripts/island.mjs";
import { en } from "../src/i18n/en.ts";
import { ru } from "../src/i18n/ru.ts";
import { browser, settled } from "./support/dom.mjs";

let Stage;
let render;
let window;

before(
  async () => {
    window = browser("https://tolearn.local/ru/stage/?program=chip&node=rom&stage=voices");
    globalThis.location = window.location;
    ({ default: Stage } = await island("Stage"));
    ({ render } = await import("solid-js/web"));
  },
  { timeout: 300_000 },
);

const SVG = "data:image/svg+xml;base64,PHN2Zy8+";
const PNG = "data:image/png;base64,iVBORw0KGgo=";

const block = (id, kind, text, extra = {}) => ({
  id,
  kind,
  text,
  lang: null,
  src: null,
  license: null,
  attribution: null,
  source: null,
  ...extra,
});

const OUT = {
  program: "chip",
  node: "chip",
  node_title: "Chiptune",
  id: "voices",
  title: "Голоса чипа",
  blocks: [
    block("h1", "heading", "Пять голосов"),
    block("p1", "paragraph", "Чип держит пять каналов:\n\n- два пульса\n- треугольник"),
    block("d1", "diagram", "Схема каналов", { src: SVG }),
    block("i1", "image", "Форма пульса", {
      src: PNG,
      license: "CC0-1.0",
      attribution: "toLearn contributors",
      source: "https://example.org/pulse",
    }),
    block("k1", "code", "print('hi')", { lang: "python" }),
    block("n1", "callout", "Громкость ограничена"),
  ],
  practice: {
    task: [block("t1", "paragraph", "Сыграйте гамму")],
    deliverable: "Файл melody.nsf",
    constraints: [{ id: "c1", claim: "Только пульсы", check: null, expect: "" }],
    acceptance: [{ id: "a1", claim: "Гамма звучит", check: "python play.py", expect: "exit 0" }],
  },
  questions: [
    { id: "q1", text: "Сколько каналов у чипа?" },
    { id: "q2", text: "Чем пульс отличается от треугольника?" },
  ],
};

function mount(options = {}) {
  const host = window.document.createElement("div");
  window.document.body.append(host);
  const calls = [];
  const call = (name, payload) => {
    calls.push({ name, payload });
    if (name !== "stage") throw new Error(`лишняя команда ${name}`);
    return options.fail ? Promise.reject(options.fail) : Promise.resolve(options.out ?? OUT);
  };
  const props = {
    text: options.text ?? ru,
    locale: "ru",
    call,
    program: "chip",
    node: "",
    stage: "voices",
    ...options.props,
  };
  render(() => Stage(props), host);
  return { host, calls };
}

test("без выбранного этапа экран говорит об этом, а не пустеет", async () => {
  const { host, calls } = mount({ props: { stage: "" } });
  await settled();

  assert.equal(calls.length, 0);
  assert.match(host.querySelector("[data-empty]").textContent, new RegExp(ru.stage.none));
  assert.equal(host.querySelector("[data-empty] a").getAttribute("href"), "/ru/");
});

test("этап читается по программе, узлу и этапу — из адреса, если не заданы", async () => {
  const given = mount();
  await settled();
  const fromUrl = mount({ props: { program: undefined, node: undefined, stage: undefined } });
  await settled();

  assert.deepEqual(given.calls[0].payload, { program: "chip", node: "", stage: "voices" });
  assert.deepEqual(fromUrl.calls[0].payload, { program: "chip", node: "rom", stage: "voices" });
});

test("каждый блок стоит под якорем своего id и в своей форме", async () => {
  const { host } = mount();
  await settled();

  const blocks = [...host.querySelectorAll("[data-stage-body] [data-block]")];
  assert.deepEqual(
    blocks.map((node) => [node.id, node.dataset.block, node.tagName]),
    [
      ["h1", "heading", "H3"],
      ["p1", "paragraph", "DIV"],
      ["d1", "diagram", "FIGURE"],
      ["i1", "image", "FIGURE"],
      ["k1", "code", "PRE"],
      ["n1", "callout", "ASIDE"],
    ],
  );
  assert.equal(host.querySelector("[data-stage-title]").textContent, "Голоса чипа");
});

test("текст идёт разметкой, код — блоком кода своего языка", async () => {
  const { host } = mount();
  await settled();

  assert.equal(host.querySelectorAll("#p1 li").length, 2, host.querySelector("#p1").innerHTML);
  const code = host.querySelector("#k1");
  assert.equal(code.dataset.lang, "python");
  assert.match(code.textContent, /print\('hi'\)/);
});

test("схема и картинка приходят только через img src, svg в DOM нет", async () => {
  const { host } = mount();
  await settled();

  assert.equal(host.querySelector("#d1 img").getAttribute("src"), SVG);
  assert.equal(host.querySelector("#d1 img").getAttribute("alt"), ru.stage.diagram, "скринридер прочтёт исходник Mermaid");
  assert.equal(host.querySelector("#i1 img").getAttribute("alt"), "Форма пульса");
  assert.equal(host.querySelector("#i1 img").getAttribute("src"), PNG);
  assert.equal(host.querySelector("svg"), null);
});

test("под картинкой стоят лицензия, атрибуция и источник — без выхода в сеть", async () => {
  const { host } = mount();
  await settled();

  const caption = host.querySelector("#i1 figcaption");
  assert.match(caption.textContent, /CC0-1\.0/);
  assert.match(caption.textContent, /toLearn contributors/);
  assert.match(caption.textContent, /example\.org\/pulse/);
  assert.equal(caption.querySelector("a"), null, "источник стал внешней ссылкой");
  assert.equal(host.querySelector("#d1 figcaption"), null);
});

test("практика и вопросы читаются без запуска и без зачёта", async () => {
  const { host } = mount();
  await settled();

  const practice = host.querySelector("[data-practice]");
  for (const seen of ["Сыграйте гамму", "Файл melody.nsf", "Только пульсы", "Гамма звучит", "python play.py", "exit 0"]) {
    assert.ok(practice.textContent.includes(seen), seen);
  }
  assert.match(practice.textContent, new RegExp(ru.stage.later));
  assert.deepEqual(
    [...host.querySelectorAll("[data-question]")].map((node) => node.id),
    ["q1", "q2"],
  );
  assert.equal(host.querySelectorAll("button:not([data-snip]), textarea").length, 0, "появился запуск или ответ");
});

test("Esc уводит к узлу этапа, а на вложенном узле — к подпрограмме", async () => {
  const root = mount();
  await settled();
  const up = root.host.querySelector("[data-up]");
  assert.equal(up.getAttribute("href"), "/ru/program/?program=chip");
  assert.equal(up.textContent, "Chiptune");

  const nested = mount({ out: { ...OUT, node: "rom", node_title: "Первый ROM" } });
  await settled();
  assert.equal(nested.host.querySelector("[data-up]").getAttribute("href"), "/ru/program/?program=chip&node=rom");
});

test("неоткрывшийся этап называет причину на языке словаря", async () => {
  const absent = { code: "stage.absent", message: "этапа `envelope` нет" };
  const russian = mount({ fail: absent }).host;
  const english = mount({ fail: absent, text: en }).host;
  const foreign = mount({ fail: { code: "library.foreign", message: "holds no file" }, text: en }).host;
  const strange = mount({ fail: { code: "library.unreadable", message: "`programs` не читается" } }).host;
  await settled();

  const reason = (host) => host.querySelector("[data-empty]").textContent;
  assert.ok(reason(russian).includes(ru.stage.absent), reason(russian));
  assert.ok(reason(english).includes(en.stage.absent), reason(english));
  assert.ok(reason(foreign).includes(en.stage.foreign), reason(foreign));
  assert.ok(reason(strange).includes("`programs` не читается"), reason(strange));
});

test("якорь в адресе прокручивает к своему блоку", async () => {
  const seen = [];
  const scroll = window.HTMLElement.prototype.scrollIntoView;
  window.HTMLElement.prototype.scrollIntoView = function () {
    seen.push(this.id);
  };
  window.location.hash = "#k1";
  mount();
  await settled();
  window.location.hash = "";
  window.HTMLElement.prototype.scrollIntoView = scroll;

  assert.deepEqual(seen, ["k1"]);
});

test("экран говорит на языке словаря", async () => {
  const { host } = mount({ text: en });
  await settled();

  assert.match(host.querySelector("[data-practice]").textContent, new RegExp(en.stage.practice));
  assert.doesNotMatch(host.textContent, new RegExp(ru.stage.questions));
});
