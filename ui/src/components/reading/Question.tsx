import { For, Show } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { AskView } from "../../ipc";
import Rich from "../rich/Rich";
import type { Copier, Words } from "../rich/Snip";

interface Props {
  ask: AskView;
  text: Dictionary;
  words: Words;
  copy?: Copier;
}

export default function Question(props: Props) {
  const stage = () => props.text.stage;
  const grade = (result: string) => {
    const said = result === "ok" ? stage().ok : result === "partial" ? stage().partial : stage().miss;
    return props.ask.missed.length > 0 ? `${said}. ${stage().missed}:` : said;
  };

  return (
    <li id={props.ask.id} data-question={props.ask.id} data-result={props.ask.result ?? undefined}>
      <Rich text={props.ask.text} words={props.words} copy={props.copy} />
      <Show when={props.ask.result}>{(result) => <p data-grade>{grade(result())}</p>}</Show>
      <Show when={props.ask.missed.length > 0}>
        <ul data-missed>
          <For each={props.ask.missed}>{(line) => <li>{line}</li>}</For>
        </ul>
      </Show>
    </li>
  );
}
