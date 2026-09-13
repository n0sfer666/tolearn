import assert from "node:assert/strict";
import test from "node:test";

import { en } from "../src/i18n/en.ts";
import { ru } from "../src/i18n/ru.ts";
import { settled } from "./support/dom.mjs";
import { OUT, PNG, SVG, stageScreen } from "./support/stage.mjs";

const { screen, mount } = stageScreen();

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
  const { HTMLElement, location } = screen.window;
  const scroll = HTMLElement.prototype.scrollIntoView;
  HTMLElement.prototype.scrollIntoView = function () {
    seen.push(this.id);
  };
  location.hash = "#k1";
  mount();
  await settled();
  location.hash = "";
  HTMLElement.prototype.scrollIntoView = scroll;

  assert.deepEqual(seen, ["k1"]);
});

test("у вопросов стоит итог последней попытки и что в ней упущено", async () => {
  const fresh = mount();
  const graded = mount({
    out: {
      ...OUT,
      questions: [
        { id: "q1", text: "Сколько каналов у чипа?", result: "ok", missed: [], draft: "" },
        {
          id: "q2",
          text: "Чем пульс отличается?",
          result: "partial",
          missed: ["форма волны", "громкость"],
          draft: "",
        },
      ],
    },
  });
  await settled();

  assert.equal(fresh.host.querySelector("[data-grade], [data-missed]"), null);
  const [q1, q2] = ["#q1", "#q2"].map((id) => graded.host.querySelector(id));
  assert.equal(q1.dataset.result, "ok");
  assert.equal(q1.querySelector("[data-grade]").textContent, ru.stage.ok);
  assert.equal(q1.querySelector("[data-missed]"), null);
  assert.equal(q2.dataset.result, "partial");
  assert.equal(q2.querySelector("[data-grade]").textContent, `${ru.stage.partial}. ${ru.stage.missed}:`);
  assert.deepEqual(
    [...q2.querySelectorAll("[data-missed] li")].map((node) => node.textContent),
    ["форма волны", "громкость"],
  );
});

test("экран говорит на языке словаря", async () => {
  const { host } = mount({ text: en });
  await settled();

  assert.match(host.querySelector("[data-practice]").textContent, new RegExp(en.stage.practice));
  assert.doesNotMatch(host.textContent, new RegExp(ru.stage.questions));
});
