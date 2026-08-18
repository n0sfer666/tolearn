import type { Dictionary } from "../i18n/ru";
import type { GenerateStateOut } from "../ipc";

type Text = Dictionary["generate"];

const PER_WORD = 3;
const TAIL_LINES = 4;

export function spell(seconds: number, text: Text): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const said: string[] = [];
  if (hours > 0) said.push(`${hours} ${text.hoursShort}`);
  if (hours > 0 || minutes > 0) said.push(`${minutes} ${text.minutes}`);
  said.push(`${seconds % 60} ${text.seconds}`);
  return said.join(" ");
}

export function crunching(live: GenerateStateOut, text: Text): string {
  const said = [`✻ ${word(live.ticks, text)}`];
  if (live.chars > 0) said.push(`${live.chars} ${text.letters}`);
  said.push(`${text.here} ${spell(live.step_seconds, text)}`);
  said.push(`${text.whole} ${spell(live.seconds, text)}`);
  return said.join(" · ");
}

export function tail(live: GenerateStateOut): string[] {
  return live.tail
    .split("\n")
    .filter((line) => line.trim() !== "")
    .slice(-TAIL_LINES);
}

function word(ticks: number, text: Text): string {
  const words = text.crunch;
  return words[Math.floor(ticks / PER_WORD) % words.length] ?? "";
}
