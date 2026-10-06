import assert from "node:assert/strict";
import test, { before, afterEach } from "node:test";

import { reveal } from "../src/lib/anchor.ts";
import { browser } from "./support/dom.mjs";

let window;

before(() => {
  window = browser("https://tolearn.local/ru/stage/");
  globalThis.location = window.location;
});

afterEach(() => {
  window.location.hash = "";
  window.document.documentElement.style.removeProperty("scroll-padding-block-start");
});

function page(body, height) {
  const document = window.document;
  document.body.innerHTML = body;
  const bar = document.querySelector(".bar");
  if (bar !== null) {
    bar.getBoundingClientRect = () => ({ x: 0, y: 0, top: 0, left: 0, right: 0, width: 0, height, bottom: height });
  }
  const seen = [];
  for (const block of document.querySelectorAll("[id]")) {
    block.scrollIntoView = () =>
      seen.push({ id: block.id, padding: document.documentElement.style.getPropertyValue("scroll-padding-block-start") });
  }
  return seen;
}

test("блок по якорю встаёт под липкую шапку, а не за неё", () => {
  const seen = page('<header class="bar"></header><p id="k1">абзац</p>', 61);
  window.location.hash = "#k1";

  reveal();

  assert.deepEqual(seen, [{ id: "k1", padding: "61px" }]);
});

test("без шапки блок встаёт к самому верху", () => {
  const seen = page('<p id="k1">абзац</p>', 0);
  window.location.hash = "#k1";

  reveal();

  assert.deepEqual(seen, [{ id: "k1", padding: "0px" }]);
});

test("без якоря в адресе ничего не прокручивается", () => {
  const seen = page('<header class="bar"></header><p id="k1">абзац</p>', 61);

  reveal();

  assert.deepEqual(seen, []);
});
