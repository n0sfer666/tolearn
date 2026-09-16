import { Show, createSignal } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import Dictate from "./Dictate";
import type { Voice } from "./voice";

interface Props {
  text: Dictionary;
  voice: Voice;
  field: string;
  fragment?: string;
  locked: boolean;
  send: (question: string) => void;
  cancel: () => void;
}

const QUESTION_CHARS = 1000;

export default function Asking(props: Props) {
  const [question, setQuestion] = createSignal("");
  const [area, setArea] = createSignal<HTMLTextAreaElement>();

  return (
    <div data-asking>
      <Show when={props.fragment}>{(fragment) => <blockquote data-fragment>{fragment()}</blockquote>}</Show>
      <label>
        {props.text.stage.clarifyQuestion}
        <textarea
          ref={setArea}
          data-doubt
          maxLength={QUESTION_CHARS}
          value={question()}
          disabled={props.locked}
          onInput={(event) => setQuestion(event.currentTarget.value)}
        />
      </label>
      <p data-asking-actions>
        <Dictate
          text={props.text}
          voice={props.voice}
          field={props.field}
          area={area}
          locked={props.locked}
          put={setQuestion}
        />
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
