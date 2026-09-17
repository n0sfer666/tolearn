import { createEffect, createSignal, onCleanup } from "solid-js";
import type { Accessor } from "solid-js";

export function elapsed(
  since: Accessor<number | null>,
  every = 1000,
  now: () => number = () => Date.now(),
): Accessor<number> {
  const [spent, setSpent] = createSignal(0);

  createEffect(() => {
    const began = since();
    if (began === null) {
      setSpent(0);
      return;
    }
    setSpent(now() - began);
    const beat = setInterval(() => setSpent(now() - began), every);
    onCleanup(() => clearInterval(beat));
  });

  return spent;
}
