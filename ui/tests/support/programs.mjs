import { before } from "node:test";

import { island } from "../../scripts/island.mjs";
import { ru } from "../../src/i18n/ru.ts";
import { browser, toasts } from "./dom.mjs";
import { held } from "./generation.mjs";

export const CHIPTUNE = "0d4f6c8a-2b1e-4f3a-9c5d-7e8f9a0b1c2d";
const span = (min, max) => ({ min, max });

export const SHELF = {
  uuid: CHIPTUNE,
  title: "Chiptune: музыка звукового чипа NES",
  goal: "Написать и проиграть мелодию на пяти голосах",
  hours: span(7, 11),
  stages: 3,
  subprograms: 0,
  summary: { passed: 0, total: 1, skipped: 0 },
  active: null,
  unread: null,
};

export const NES = {
  uuid: "nes-dev",
  title: "Разработка игр для NES",
  goal: "Собрать игру",
  hours: span(14, 22),
  stages: 0,
  subprograms: 2,
  summary: { passed: 1, total: 2, skipped: 0 },
  active: "2026-09-10",
  unread: null,
};

export const BROKEN = { directory: "5f0e", code: "library.malformed", message: "program.yaml не читается" };

export function programsScreen() {
  const screen = { window: null };
  let Programs;
  let render;

  before(
    async () => {
      screen.window = browser();
      ({ default: Programs } = await island("Programs"));
      ({ render } = await import("solid-js/web"));
    },
    { timeout: 300_000 },
  );

  const mount = (options = {}) => {
    const host = screen.window.document.createElement("div");
    screen.window.document.body.append(host);
    const calls = [];
    let drop = () => {};
    const call = (name, payload) => {
      calls.push({ name, payload });
      if (name === "library") {
        return Promise.resolve({ programs: options.listing ?? [SHELF], refused: options.refused ?? [] });
      }
      if (name !== "import_package") throw new Error(`лишняя команда ${name}`);
      if (options.hold) return held();
      if (options.fail) return Promise.reject(options.fail);
      return Promise.resolve(options.imported ?? { uuid: CHIPTUNE, title: SHELF.title, copy_of: null });
    };
    const said = toasts(screen.window);
    const props = {
      text: ru.programs,
      locale: options.locale ?? "ru",
      call,
      pick: options.pick ?? (() => Promise.resolve("/incoming/chiptune.tolearn")),
      drops: (handler) => {
        drop = handler;
      },
    };
    const dispose = render(() => Programs(props), host);
    return { host, calls, said, dispose, drop: (paths) => drop(paths) };
  };

  return { mount };
}
