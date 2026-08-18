import { Show } from "solid-js";

import type { Dictionary } from "../../i18n/ru";

interface Props {
  text: Dictionary["generate"];
  enabled: boolean | null;
  onOpen: () => void;
}

export default function Tile(props: Props) {
  return (
    <>
      <button
        type="button"
        data-generate-open
        aria-label={props.text.add}
        title={props.text.add}
        disabled={props.enabled !== true}
        onClick={props.onOpen}
      >
        <span aria-hidden="true">+</span>
      </button>
      <Show when={props.enabled === false}>
        <p data-generate-off>{props.text.off}</p>
      </Show>
    </>
  );
}
