import { For, Show } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { SummaryView } from "../../ipc";

interface Props {
  text: Dictionary["generate"];
  summary: SummaryView;
  busy: boolean;
  refused: string[];
  onAccept: () => void;
  onClose: () => void;
}

export default function Ready(props: Props) {
  const hours = () => {
    const { hours_min: least, hours_max: most } = props.summary;
    return least === most ? `${least}` : `${least}–${most}`;
  };

  return (
    <div data-generate-summary>
      <h3>{props.text.summary}</h3>
      <p data-generate-title>{props.summary.title}</p>
      <p data-generate-total>
        {props.summary.topics} {props.text.topics}, {hours()} {props.text.hours}
      </p>
      <ol data-generate-stages>
        <For each={props.summary.stages}>
          {(stage) => (
            <li>
              {stage.title} — {stage.topics} {props.text.topics}
              <ul>
                <For each={stage.first}>{(title) => <li>{title}</li>}</For>
              </ul>
            </li>
          )}
        </For>
      </ol>
      <Show when={props.refused.length > 0}>
        <div data-generate-refused role="alert">
          <h4>{props.text.refused}</h4>
          <ul>
            <For each={props.refused}>{(why) => <li>{why}</li>}</For>
          </ul>
        </div>
      </Show>
      <div class="row">
        <button type="button" data-generate-accept disabled={props.busy} onClick={props.onAccept}>
          {props.busy ? props.text.accepting : props.text.accept}
        </button>
        <button type="button" data-generate-close onClick={props.onClose}>
          {props.text.cancel}
        </button>
      </div>
    </div>
  );
}
