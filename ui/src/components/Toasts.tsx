import { For, createSignal, onCleanup, onMount } from "solid-js";

import type { Said, Tone } from "../lib/toast";
import { TOASTED, said } from "../lib/toast";

interface Props {
  close: string;
}

interface Shown extends Said {
  id: number;
}

const LIFE: Record<Tone, number> = { ok: 3000, info: 4000, warn: 6000, error: 9000 };

export default function Toasts(props: Props) {
  const [shown, setShown] = createSignal<readonly Shown[]>([]);
  let counted = 0;

  const drop = (id: number) => setShown((kept) => kept.filter((note) => note.id !== id));

  const listen = (event: Event) => {
    const note = said(event);
    if (note === null) return;
    counted += 1;
    setShown((kept) => [...kept, { ...note, id: counted }]);
  };

  onMount(() => {
    window.addEventListener(TOASTED, listen);
    onCleanup(() => window.removeEventListener(TOASTED, listen));
  });

  return (
    <ul data-toasts>
      <For each={shown()}>
        {(note) => <Row note={note} close={props.close} onDrop={() => drop(note.id)} />}
      </For>
    </ul>
  );
}

interface RowProps {
  note: Shown;
  close: string;
  onDrop: () => void;
}

function Row(props: RowProps) {
  onMount(() => {
    const timer = setTimeout(props.onDrop, LIFE[props.note.tone]);
    onCleanup(() => clearTimeout(timer));
  });

  return (
    <li data-toast={props.note.tone} role={loud(props.note.tone) ? "alert" : "status"}>
      <span data-toast-text>{props.note.text}</span>
      <button type="button" data-toast-close aria-label={props.close} onClick={props.onDrop}>
        ✕
      </button>
    </li>
  );
}

function loud(tone: Tone): boolean {
  return tone === "error" || tone === "warn";
}
