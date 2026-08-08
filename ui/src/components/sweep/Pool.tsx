import { For, Show } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { SweepPickView } from "../../ipc";

interface Props {
  text: Dictionary["sweep"];
  ready: SweepPickView[];
  topics: number;
  busy: boolean;
  onTopics: (topics: number) => void;
  onStart: () => void;
}

export default function Pool(props: Props) {
  return (
    <section data-pool>
      <Show when={props.ready.length > 0} fallback={<p data-empty>{props.text.empty}</p>}>
        <h3>{props.text.pool}</h3>
        <ul>
          <For each={props.ready}>
            {(pick) => (
              <li data-pick={pick.topic} data-overdue={pick.overdue ? "" : undefined}>
                <span>{pick.title}</span>
                <Show when={pick.due}>
                  {(due) => (
                    <span data-due>
                      {pick.overdue ? props.text.overdue : props.text.due}: {due()}
                    </span>
                  )}
                </Show>
              </li>
            )}
          </For>
        </ul>
        <p class="row">
          <label>
            {props.text.topics}
            <input
              type="number"
              data-topics
              min={1}
              max={props.ready.length}
              value={props.topics}
              onInput={(event) => props.onTopics(Number(event.currentTarget.value))}
            />
          </label>
          <button type="button" data-start disabled={props.busy} onClick={props.onStart}>
            {props.text.start}
          </button>
        </p>
      </Show>
    </section>
  );
}
