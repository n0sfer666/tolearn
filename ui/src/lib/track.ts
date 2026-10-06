import type { Dictionary } from "../i18n/ru";

export function track(text: Dictionary) {
  return [
    { key: "asked", label: text.generate.markAsked },
    { key: "reach", label: text.generate.markReach },
    { key: "drawn", label: text.generate.markDrawn },
  ];
}
