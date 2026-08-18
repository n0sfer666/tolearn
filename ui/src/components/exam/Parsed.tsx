import { For, Show } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { VerdictView } from "../../ipc";
import { label } from "../status";

interface Props {
  text: Dictionary;
  verdict: VerdictView;
}

export default function Parsed(props: Props) {
  return (
    <section data-parsed>
      <h3>{props.text.exam.parsed}</h3>
      <p data-status>{label(props.text, props.verdict.status)}</p>
      <Show when={props.verdict.gaps.length > 0}>
        <ul data-gaps>
          <For each={props.verdict.gaps}>{(gap) => <li>{gap}</li>}</For>
        </ul>
      </Show>
      <ul data-answers>
        <For each={props.verdict.per_question}>
          {(answer) => (
            <li data-answer={answer.id}>
              {answer.id}: {answer.outcome}
            </li>
          )}
        </For>
      </ul>
      <Show when={props.verdict.missing.length > 0}>
        <p data-missing>
          {props.text.exam.missing}: {props.verdict.missing.join(", ")}
        </p>
      </Show>
      <Show when={props.verdict.unknown_questions.length > 0}>
        <p data-unknown>
          {props.text.exam.unknown}: {props.verdict.unknown_questions.join(", ")}
        </p>
      </Show>
    </section>
  );
}
