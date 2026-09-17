import { Show } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import { clocked } from "../../lib/clocked";
import type { Generation } from "../../lib/generation";
import { grab } from "../../lib/grab";

const PATIENCE = 60_000;

interface Props {
  text: Dictionary;
  work: Generation;
}

export default function Progress(props: Props) {
  return (
    <div data-progress role="status">
      <p data-said>
        <span data-spin aria-hidden="true" />
        <span data-step>{props.work.said()}</span>
        <span data-elapsed>{clocked(props.work.spent(), props.text)}</span>
      </p>
      <button
        type="button"
        data-cancel
        ref={grab}
        aria-disabled={props.work.cancelling() ? "true" : undefined}
        onClick={() => props.work.cancel()}
      >
        {props.text.generate.cancel}
      </button>
      <Show when={props.work.spent() >= PATIENCE}>
        <p data-patience>{props.text.generate.patience}</p>
      </Show>
    </div>
  );
}
