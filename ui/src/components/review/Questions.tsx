import { For, Show } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { ReviewedView } from "../../ipc";
import { outcome } from "./labels";

interface Props {
  text: Dictionary;
  questions: ReviewedView[];
}

export default function Questions(props: Props) {
  return (
    <ul data-questions>
      <For each={props.questions}>
        {(question) => (
          <li data-question={question.id}>
            <p>
              {question.id} ({question.kind})
              <Show when={question.outcome}>
                {(result) => <span data-outcome>{outcome(props.text, result())}</span>}
              </Show>
            </p>
            <p>{question.text}</p>
            <Show when={question.missed.length > 0}>
              <p data-missed>
                {props.text.review.missed}: {question.missed.join(", ")}
              </p>
            </Show>
          </li>
        )}
      </For>
    </ul>
  );
}
