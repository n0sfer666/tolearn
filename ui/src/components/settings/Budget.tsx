import { createEffect, createSignal } from "solid-js";

import type { Dictionary } from "../../i18n/ru";

interface Props {
  text: Dictionary;
  value: number;
  onPick: (value: string) => void;
}

const MIN = 256;
const MAX = 16384;
const STEP = 256;

export default function Budget(props: Props) {
  const [shown, setShown] = createSignal(props.value);

  createEffect(() => setShown(props.value));

  const move = (raw: string) => {
    const value = Number.parseInt(raw, 10);
    if (Number.isNaN(value)) return;
    setShown(value);
  };

  return (
    <div class="row">
      <input
        type="range"
        data-budget-slider
        min={MIN}
        max={MAX}
        step={STEP}
        aria-label={props.text.settings.budget}
        value={shown()}
        onInput={(event) => move(event.currentTarget.value)}
        onChange={(event) => props.onPick(event.currentTarget.value)}
      />
      <input
        data-budget
        type="number"
        min="1"
        aria-label={props.text.settings.budget}
        value={shown()}
        onInput={(event) => move(event.currentTarget.value)}
        onChange={(event) => props.onPick(event.currentTarget.value)}
      />
    </div>
  );
}
