import { after, before } from "node:test";

import { island } from "../../scripts/island.mjs";
import { ru } from "../../src/i18n/ru.ts";
import { browser, settled, toasts } from "./dom.mjs";
import { heard, press, transport } from "./generation.mjs";

export const REQUEST = "Хочу писать чиптюн";
export const LEVEL = "Нот не знаю";
const span = (min, max) => ({ min, max });
export const PLAN = {
  plan: {
    title: "Чиптюн с нуля",
    slug: "chiptune",
    goal: "Написать трек на пяти голосах",
    volatility: "stable",
    stages: [
      { id: "tracker", title: "Трекер и паттерны", hours: span(2, 3) },
      { id: "voices", title: "Голоса чипа", hours: span(3, 4) },
    ],
    children: [{ title: "Сведение", goal: "Свести трек", hours: span(4, 6) }],
  },
  hours: span(9, 13),
};
export const REVISED = { plan: { ...PLAN.plan, title: "Чиптюн без трекера" }, hours: span(5, 7) };
export const SPLIT = {
  plan: {
    ...PLAN.plan,
    stages: [],
    children: [
      { title: "Трекер и паттерны", goal: "Собрать первый паттерн", hours: span(40, 60) },
      { title: "Сведение", goal: "Свести трек", hours: span(50, 90) },
    ],
  },
  hours: span(90, 150),
};
export const LOG = "/tmp/tolearn/llm";
export const STARTED = { program: "chip", node: "chip", stage: "tracker" };

export function fill(host, selector, value) {
  const field = host.querySelector(selector);
  field.value = value;
  field.dispatchEvent(new Event("input", { bubbles: true }));
}

export function newScreen() {
  const screen = { window: null };
  const alive = [];
  after(() => {
    for (const dispose of alive) dispose();
  });
  let New;
  let render;

  before(
    async () => {
      screen.window = browser("https://tolearn.local/ru/new/");
      ({ default: New } = await island("New"));
      ({ render } = await import("solid-js/web"));
    },
    { timeout: 300_000 },
  );

  const mount = (options = {}) => {
    const host = screen.window.document.createElement("div");
    screen.window.document.body.append(host);
    const gone = [];
    const { calls, call } = transport({
      plan_program: PLAN,
      revise_plan: REVISED,
      start_program: STARTED,
      cancel_generation: { cancelled: true },
      llm_log: (payload) => Promise.resolve({ room: LOG, records: payload.clear ? 0 : 3 }),
      ...options.answers,
    });
    const said = toasts(screen.window);
    const { steps, emit, stops } = heard();
    const props = { text: options.text ?? ru, locale: options.locale ?? "ru", call, steps, go: (href) => gone.push(href) };
    const dispose = render(() => New(props), host);
    alive.push(dispose);
    return { host, calls, said, gone, dispose, emit, stops };
  };

  const planned = async (options = {}) => {
    const mounted = mount(options);
    fill(mounted.host, "[data-request]", REQUEST);
    fill(mounted.host, "[data-level]", LEVEL);
    press(mounted.host, "[data-plan]");
    await settled();
    return mounted;
  };

  return { screen, mount, planned };
}
