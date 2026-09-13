import { Show } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import { grab } from "../../lib/grab";
import type { Refused } from "../../lib/refusal";

interface Props {
  text: Dictionary;
  locale: string;
  refused: Refused | null;
}

export default function Refusal(props: Props) {
  return (
    <Show when={props.refused} keyed>
      {(refused) => (
        <div data-refused role="alert" tabIndex={-1} ref={grab}>
          <Show when={refused.failed}>
            <h3>{props.text.generate.refused}</h3>
          </Show>
          <p>{refused.reason}</p>
          <Show when={refused.settings}>
            <a href={`/${props.locale}/settings/`} data-to-settings>
              {props.text.generate.settings}
            </a>
          </Show>
        </div>
      )}
    </Show>
  );
}
