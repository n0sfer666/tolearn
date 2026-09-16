import { Show, createSignal, onCleanup, onMount } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import { type Aim, type Spot, focused, picked, spotted } from "../../lib/picked";

interface Props {
  text: Dictionary;
  body: () => HTMLElement | undefined;
  pick: (aim: Aim) => void;
}

const PARKED = "[data-actions], [data-pick]";

export default function Pick(props: Props) {
  const [aim, setAim] = createSignal<Aim | null>(null);
  const [spot, setSpot] = createSignal<Spot>({ top: 0, left: 0, below: false });
  let range: Range | null = null;
  let host: HTMLElement | null = null;

  const place = () => {
    const rect = range?.getBoundingClientRect() ?? host?.getBoundingClientRect();
    if (rect !== undefined) setSpot(spotted(rect, window.innerWidth));
  };

  const found = (aimed: Aim, at: Range | null, block: HTMLElement | null) => {
    range = at;
    host = block;
    setAim(aimed);
    place();
  };

  const dropped = () => {
    range = null;
    host = null;
    setAim(null);
  };

  const refresh = () => {
    const body = props.body();
    if (body === undefined) return;
    const selected = picked(document, body);
    if (selected !== null) return found(selected.aim, selected.range, null);
    const block = focused(document, body);
    if (block !== null) return found({ block: block.id, fragment: "" }, null, block);
    if (document.activeElement?.closest(PARKED) != null) return;
    dropped();
  };

  const tapped = (event: Event) => {
    if (event.target instanceof Element && event.target.closest(PARKED) !== null) return;
    setTimeout(refresh, 0);
  };

  onMount(() => {
    document.addEventListener("selectionchange", refresh);
    document.addEventListener("focusin", refresh);
    document.addEventListener("pointerdown", tapped);
    window.addEventListener("scroll", place, true);
    onCleanup(() => {
      document.removeEventListener("selectionchange", refresh);
      document.removeEventListener("focusin", refresh);
      document.removeEventListener("pointerdown", tapped);
      window.removeEventListener("scroll", place, true);
    });
  });

  const chose = () => {
    const aimed = aim();
    if (aimed === null) return;
    dropped();
    props.pick(aimed);
  };

  return (
    <Show when={aim()}>
      {(aimed) => (
        <button
          type="button"
          data-pick={aimed().block}
          data-action
          data-below={spot().below ? "" : undefined}
          style={{ top: `${spot().top}px`, left: `${spot().left}px` }}
          onMouseDown={(event) => event.preventDefault()}
          onClick={chose}
        >
          {aimed().fragment === "" ? props.text.stage.clarifyBlock : props.text.stage.clarify}
        </button>
      )}
    </Show>
  );
}
