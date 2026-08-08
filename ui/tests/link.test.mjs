import assert from "node:assert/strict";
import test, { before } from "node:test";

import { island } from "../scripts/island.mjs";
import { ru } from "../src/i18n/ru.ts";
import { browser, settled, toasts } from "./support/dom.mjs";

let landing;
let Programs;
let render;
let window;

before(async () => {
  window = browser("https://tolearn.local/ru/");
  globalThis.location = window.location;
  ({ landing } = await island("landing", "src/lib", "ts"));
  ({ default: Programs } = await island("Programs"));
  ({ render } = await import("solid-js/web"));
}, { timeout: 300_000 });

function mount() {
  const host = window.document.createElement("div");
  window.document.body.append(host);
  const call = (name) => {
    if (name === "programs") return Promise.resolve({ programs: [] });
    if (name === "provider") return Promise.resolve({ provider: { enabled: true } });
    return new Promise(() => {});
  };
  return render(
    () =>
      Programs({
        text: ru.programs,
        generate: ru.generate,
        locale: "ru",
        call,
        pick: () => Promise.resolve(null),
        pickArchive: () => Promise.resolve(null),
        drops: () => {},
      }),
    host,
  );
}

test("ссылка на тему ведёт прямо на тему выбранного языка", () => {
  assert.equal(
    landing("ru", "?program=%2Fprograms%2Fbase&topic=agents"),
    "/ru/topic/?program=%2Fprograms%2Fbase&topic=agents",
  );
});

test("половина адреса не считается адресом темы", () => {
  assert.equal(landing("en", "?program=%2Fprograms%2Fbase"), "/en/");
  assert.equal(landing("en", "?topic=agents"), "/en/");
  assert.equal(landing("ru", ""), "/ru/");
});

test("отказ доезжает до списка программ, а не теряется по дороге", () => {
  assert.equal(
    landing("ru", "?refused=%D0%BD%D0%B5%D1%82%20%D1%82%D0%B5%D0%BC%D1%8B"),
    "/ru/?refused=%D0%BD%D0%B5%D1%82+%D1%82%D0%B5%D0%BC%D1%8B",
  );
});

test("список программ произносит отказ вслух", async () => {
  window.happyDOM.setURL("https://tolearn.local/ru/?refused=программа%20не%20в%20реестре");
  const said = toasts(window);

  const dispose = mount();
  await settled();
  dispose();

  assert.deepEqual(said, [{ tone: "error", text: "программа не в реестре" }]);
});

test("обычный вход в список молчит", async () => {
  window.happyDOM.setURL("https://tolearn.local/ru/");
  const said = toasts(window);

  const dispose = mount();
  await settled();
  dispose();

  assert.deepEqual(said, []);
});
