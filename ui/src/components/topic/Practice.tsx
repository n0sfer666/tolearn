import { For, Show } from "solid-js";

import type { CheckView, PracticeView } from "../../ipc";
import type { Dictionary } from "../../i18n/ru";

interface Props {
  text: Dictionary;
  practice: PracticeView;
}

function checks(items: CheckView[]) {
  return (
    <ul>
      <For each={items}>
        {(item) => (
          <li data-check={item.id}>
            <span data-claim>{item.claim}</span>
            <code>{item.check}</code>
            <span data-expect>{item.expect}</span>
          </li>
        )}
      </For>
    </ul>
  );
}

export default function Practice(props: Props) {
  return (
    <>
      <p data-task>{props.practice.task}</p>
      <p data-deliverable>
        {props.text.topic.deliverable}: {props.practice.deliverable}
      </p>
      <p data-box>
        {props.text.topic.timeBox}: {props.practice.time_box_min} {props.text.topic.minutes}
        <Show when={props.practice.smoke_checked}>
          <span data-smoke>{props.text.topic.smoke}</span>
        </Show>
      </p>
      <div data-constraints>
        <h3>{props.text.topic.constraints}</h3>
        {checks(props.practice.constraints)}
      </div>
      <div data-acceptance>
        <h3>{props.text.topic.acceptance}</h3>
        {checks(props.practice.acceptance)}
      </div>
      <Show when={props.practice.fallback}>
        {(hint) => (
          <details>
            <summary>{props.text.topic.hint}</summary>
            <p>{hint()}</p>
          </details>
        )}
      </Show>
    </>
  );
}
