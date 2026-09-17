import { after, before } from "node:test";

import { island } from "../../scripts/island.mjs";
import { ru } from "../../src/i18n/ru.ts";
import { browser, toasts } from "./dom.mjs";
import { heard, transport } from "./generation.mjs";

const span = (min, max) => ({ min, max });

export const FORK = {
  variants: [
    { id: "noise", title: "Шумовой канал", hours: span(2, 3), why: "Без шума нет барабанов", recommended: true },
    { id: "dpcm", title: "Сэмплы DPCM", hours: span(3, 5), why: "Живые барабаны", recommended: false },
  ],
};

const row = (id, title, status, pass, ready = true) => ({ id, title, hours: span(2, 3), ready, status, pass });

export const NODE = {
  program: "chip",
  uuid: "rom",
  title: "ROM и звук",
  goal: "Собрать трек на пяти голосах",
  level: "Нот не знаю",
  hours: span(7, 10),
  trail: [{ uuid: "chip", title: "Чиптюн с нуля" }],
  stages: [
    row("pulse", "Прямоугольник", "passed", "exam"),
    row("voices", "Голоса чипа", "passed", "exam"),
    row("ahead", "Следующий этап", "fresh", null, false),
  ],
  children: [],
  summary: { passed: 2, total: 3, skipped: 0 },
  sources: { books: [], pages: [] },
};

const claim = (id) => ({ id, claim: `Пункт ${id}`, check: null, expect: "" });
const ask = (id, text, result, missed = []) => ({ id, text, result, missed, answer: null, draft: "" });

export const STAGE = {
  program: "chip",
  node: "rom",
  node_title: "ROM и звук",
  id: "voices",
  title: "Голоса чипа",
  blocks: [],
  practice: { task: [], deliverable: "Патч на пять голосов", constraints: [claim("c1")], acceptance: [claim("a1")] },
  questions: [ask("q1", "Сколько голосов у чипа?", "ok"), ask("q2", "Что делает канал шума?", "miss", ["про DPCM"])],
  ticks: ["c1"],
  workdir: null,
  clarifications: [
    { chain: 1, block: "b1", excerpt: "голоса", fragment: null, turns: [], clear: true },
    { chain: 2, block: "b2", excerpt: "шум", fragment: null, turns: [], clear: false },
  ],
};

export function forkScreen() {
  const alive = [];
  after(() => {
    for (const dispose of alive) dispose();
  });
  const screen = { window: null };
  let Next;
  let render;

  before(
    async () => {
      screen.window = browser("https://tolearn.local/ru/next/?program=chip&node=rom&stage=voices");
      globalThis.location = screen.window.location;
      ({ default: Next } = await island("Next"));
      ({ render } = await import("solid-js/web"));
    },
    { timeout: 300_000 },
  );

  const mount = (options = {}) => {
    const host = screen.window.document.createElement("div");
    screen.window.document.body.append(host);
    const gone = [];
    const { calls, call } = transport({
      fork: FORK,
      node: NODE,
      stage: STAGE,
      take_next: { node: "rom", stage: "dpcm" },
      cancel_generation: { cancelled: true },
      ...options.answers,
    });
    const said = toasts(screen.window);
    const { steps, emit } = heard();
    const props = {
      text: options.text ?? ru,
      locale: options.locale ?? "ru",
      call,
      steps,
      go: (href) => gone.push(href),
      ...options.props,
    };
    const dispose = render(() => Next(props), host);
    alive.push(dispose);
    return { host, calls, said, gone, dispose, emit };
  };

  const focus = (host, selector) => {
    const event = new screen.window.Event("focusin", { bubbles: true });
    host.querySelector(selector).dispatchEvent(event);
  };

  const hover = (host, selector) => {
    const event = new screen.window.Event("mouseover", { bubbles: true });
    host.querySelector(selector).dispatchEvent(event);
  };

  const leave = (host, selector) => {
    host.querySelector(selector).dispatchEvent(new screen.window.Event("mouseleave"));
  };

  return { screen, mount, focus, hover, leave };
}
