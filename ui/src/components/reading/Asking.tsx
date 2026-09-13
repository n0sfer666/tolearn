import { createSignal } from "solid-js";

import type { Dictionary } from "../../i18n/ru";

interface Props {
  text: Dictionary;
  locked: boolean;
  send: (question: string) => void;
  cancel: () => void;
}

const QUESTION_CHARS = 1000;

export default function Asking(props: Props) {
  const [question, setQuestion] = createSignal("");

  return (
    <div data-asking>
      <label>
        {props.text.stage.clarifyQuestion}
        <textarea
          data-doubt
          maxLength={QUESTION_CHARS}
          value={question()}
          disabled={props.locked}
          onInput={(event) => setQuestion(event.currentTarget.value)}
        />
      </label>
      <p data-asking-actions>
        <button type="button" data-ask disabled={props.locked} onClick={() => props.send(question())}>
          {props.text.stage.clarifyAsk}
        </button>
        <button type="button" data-unask disabled={props.locked} onClick={() => props.cancel()}>
          {props.text.stage.clarifyCancel}
        </button>
      </p>
    </div>
  );
}
