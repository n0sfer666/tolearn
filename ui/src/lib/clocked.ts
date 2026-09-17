import type { Dictionary } from "../i18n/ru";

const MINUTE = 60;

export function clocked(spent: number, text: Dictionary): string {
  const whole = Math.max(0, Math.floor(spent / 1000));
  if (whole < MINUTE) return text.generate.seconds.replace("{n}", String(whole));
  return `${Math.floor(whole / MINUTE)}:${String(whole % MINUTE).padStart(2, "0")}`;
}
