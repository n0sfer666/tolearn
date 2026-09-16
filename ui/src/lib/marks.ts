import { createStore } from "solid-js/store";

export interface Step {
  key: string;
  label: string;
}

export interface Mark extends Step {
  done: boolean;
  spent: number | null;
}

export interface Marks {
  seen: Mark[];
  start: (track: Step[]) => void;
  note: (key: string) => void;
  hit: (key: string) => void;
}

export function marks(now: () => number = () => Date.now()): Marks {
  const [seen, setSeen] = createStore<Mark[]>([]);
  const stamped = new Set<string>();
  let order: string[] = [];
  let last = 0;

  const start = (track: Step[]) => {
    stamped.clear();
    order = track.map((step) => step.key);
    last = now();
    setSeen(track.map((step) => ({ ...step, done: false, spent: null })));
  };

  const stamp = (key: string, spent: number | null): boolean => {
    const at = order.indexOf(key);
    if (at < 0 || stamped.has(key)) return false;
    stamped.add(key);
    setSeen(at, { done: true, spent });
    return true;
  };

  const note = (key: string) => void stamp(key, null);

  const hit = (key: string) => {
    const moment = now();
    if (!stamp(key, moment - last)) return;
    last = moment;
  };

  return { seen, start, note, hit };
}
