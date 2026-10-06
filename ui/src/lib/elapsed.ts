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
    const tick = () => setSpent(now() - began);
    tick();
    let beat: ReturnType<typeof setInterval> | undefined;
    const first = setTimeout(
      () => {
        tick();
        beat = setInterval(tick, every);
      },
      every - ((now() - began) % every),
    );
    onCleanup(() => {
      clearTimeout(first);
      clearInterval(beat);
    });
  });

  return spent;
}
