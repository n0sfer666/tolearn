import { Show } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { ExamStateOut } from "../../ipc";

interface Props {
  text: Dictionary["dialog"];
  view: ExamStateOut;
  draft: string;
  busy: boolean;
  onDraft: (text: string) => void;
  onSay: () => void;
  onHint: () => void;
  onFinish: () => void;
  onRestart: () => void;
}

export default function Reply(props: Props) {
  const over = () => props.view.stage === "over";

  return (
    <section data-reply>
      <p data-count>
        {props.text.asked}: {props.view.asked} / {props.view.total}
      </p>
      <Show when={over()}>
        <p data-over>{props.text.over}</p>
      </Show>
      <label>
        {props.text.answer}
        <textarea
          data-answer
          rows={6}
          disabled={props.busy || over()}
          value={props.draft}
          onInput={(event) => props.onDraft(event.currentTarget.value)}
        />
      </label>
      <p class="row">
        <button
          type="button"
          data-say
          disabled={props.busy || over() || props.draft.trim() === ""}
          onClick={props.onSay}
        >
          {props.busy ? props.text.saying : props.text.say}
        </button>
        <button
          type="button"
          data-hint
          disabled={props.busy || !props.view.hint_ready}
          onClick={props.onHint}
        >
          {props.text.hint}
        </button>
        <button type="button" data-finish disabled={props.busy} onClick={props.onFinish}>
          {props.text.finish}
        </button>
        <button type="button" data-restart disabled={props.busy} onClick={props.onRestart}>
          {props.text.restart}
        </button>
      </p>
      <p data-cost>
        {props.text.cost}: {props.view.seconds} {props.text.seconds}, {props.view.tokens}{" "}
        {props.text.tokens}
      </p>
    </section>
  );
}
