import { For, Show } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import { hours } from "../../lib/hours";
import type { Ranked } from "../../lib/ranked";

interface Props {
  text: Dictionary;
  picks: Ranked[];
  picked: number;
  place: string | null;
  choose: (choice: number) => void;
  attend: (at: number) => void;
  hover: (at: number | null) => void;
  focus: (element: HTMLElement) => void;
}

export default function Variants(props: Props) {
  return (
    <ul data-variants tabIndex={-1} ref={props.focus} onMouseLeave={() => props.hover(null)}>
      <For each={props.picks}>
        {(pick, at) => (
          <li
            data-variant={pick.variant.id}
            data-recommended={pick.variant.recommended ? "" : undefined}
            data-picked={at() === props.picked ? "" : undefined}
            onFocusIn={() => props.attend(at())}
            onMouseOver={() => props.hover(at())}
          >
            <strong>{pick.variant.title}</strong>
            <Show when={pick.variant.recommended}>
              <span data-kind>{props.text.generate.recommended}</span>
            </Show>
            <span data-hours>{hours(pick.variant.hours, props.text.program.hours)}</span>
            <Show when={props.place}>{(label) => <span data-place>{label()}</span>}</Show>
            <p>{pick.variant.why}</p>
            <button
              type="button"
              data-choose
              aria-label={`${props.text.generate.choose}: ${pick.variant.title}`}
              onClick={() => props.choose(pick.choice)}
            >
              {props.text.generate.choose}
            </button>
          </li>
        )}
      </For>
    </ul>
  );
}
