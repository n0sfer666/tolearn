import { For, Show, createSignal } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { AskView } from "../../ipc";
import Rich from "../rich/Rich";
import type { Copier, Words } from "../rich/Snip";
import Dictate from "./Dictate";
import type { Voice } from "./voice";

interface Props {
  ask: AskView;
  text: Dictionary;
  words: Words;
  copy?: Copier;
  answer: string;
  voice: Voice;
  locked: boolean;
  onType: (text: string) => void;
  onLeave: () => void;
}

export default function Question(props: Props) {
  const [area, setArea] = createSignal<HTMLTextAreaElement>();
  const stage = () => props.text.stage;
  const grade = (result: string) => {
    const said = result === "ok" ? stage().ok : result === "partial" ? stage().partial : stage().miss;
    return props.ask.missed.length > 0 ? `${said}. ${stage().missed}:` : said;
  };

  return (
    <li id={props.ask.id} data-question={props.ask.id} data-result={props.ask.result ?? undefined}>
      <Rich text={props.ask.text} words={props.words} copy={props.copy} />
      <label>
        {stage().answer}
        <textarea
          ref={setArea}
          data-answer
          rows="3"
          readOnly={props.locked}
          value={props.answer}
          onInput={(event) => props.onType(event.currentTarget.value)}
          onBlur={() => props.onLeave()}
        />
      </label>
      <Dictate
        text={props.text}
        voice={props.voice}
        field={`question:${props.ask.id}`}
        area={area}
        locked={props.locked}
        put={props.onType}
      />
      <Show when={props.ask.result}>{(result) => <p data-grade>{grade(result())}</p>}</Show>
      <Show when={props.ask.missed.length > 0}>
        <ul data-missed>
          <For each={props.ask.missed}>{(line) => <li>{line}</li>}</For>
        </ul>
      </Show>
    </li>
  );
}
