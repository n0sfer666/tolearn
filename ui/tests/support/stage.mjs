import { before } from "node:test";

import { island } from "../../scripts/island.mjs";
import { ru } from "../../src/i18n/ru.ts";
import { browser } from "./dom.mjs";

export const SVG = "data:image/svg+xml;base64,PHN2Zy8+";
export const PNG = "data:image/png;base64,iVBORw0KGgo=";

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

export const OUT = {
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
    { id: "q1", text: "Сколько каналов у чипа?", result: null, missed: [] },
    { id: "q2", text: "Чем пульс отличается от треугольника?", result: null, missed: [] },
  ],
  ticks: [],
  workdir: null,
};

export function stageScreen() {
  const screen = { window: null };
  let Stage;
  let render;

  before(
    async () => {
      screen.window = browser("https://tolearn.local/ru/stage/?program=chip&node=rom&stage=voices");
      globalThis.location = screen.window.location;
      ({ default: Stage } = await island("Stage"));
      ({ render } = await import("solid-js/web"));
    },
    { timeout: 300_000 },
  );

  const mount = (options = {}) => {
    const host = screen.window.document.createElement("div");
    screen.window.document.body.append(host);
    const calls = [];
    const picks = [];
    const call = (name, payload) => {
      calls.push({ name, payload });
      if (name === "stage") return options.fail ? Promise.reject(options.fail) : Promise.resolve(options.out ?? OUT);
      const reply = options.replies?.[name];
      if (reply === undefined) return Promise.reject(new Error(`лишняя команда ${name}`));
      return Promise.resolve().then(() => reply(payload));
    };
    const pick = () => {
      picks.push(true);
      return Promise.resolve(options.folder ?? null);
    };
    const props = {
      text: options.text ?? ru,
      locale: "ru",
      call,
      pick,
      program: "chip",
      node: "",
      stage: "voices",
      ...options.props,
    };
    render(() => Stage(props), host);
    return { host, calls, picks };
  };

  return { screen, mount };
}
