import { For, Show } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { VariantView } from "../../ipc";
import { hours } from "../../lib/hours";

interface Props {
  text: Dictionary;
  variants: VariantView[];
  choose: (choice: number) => void;
  focus: (element: HTMLElement) => void;
}

export default function Variants(props: Props) {
  return (
    <ul data-variants tabIndex={-1} ref={props.focus}>
      <For each={props.variants}>
        {(variant, index) => (
          <li data-variant={variant.id} data-recommended={variant.recommended ? "" : undefined}>
            <strong>{variant.title}</strong>
            <Show when={variant.recommended}>
              <span data-kind>{props.text.generate.recommended}</span>
            </Show>
            <span data-hours>{hours(variant.hours, props.text.program.hours)}</span>
            <p>{variant.why}</p>
            <button
              type="button"
              data-choose
              aria-label={`${props.text.generate.choose}: ${variant.title}`}
              onClick={() => props.choose(index())}
            >
              {props.text.generate.choose}
            </button>
          </li>
        )}
      </For>
    </ul>
  );
}
