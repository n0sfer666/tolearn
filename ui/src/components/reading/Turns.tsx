import { For, Show } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { TurnView } from "../../ipc";
import Rich from "../rich/Rich";
import type { Copier, Words } from "../rich/Snip";

interface Props {
  text: Dictionary;
  turns: TurnView[];
  words: Words;
  copy?: Copier;
}

export default function Turns(props: Props) {
  return (
    <For each={props.turns}>
      {(turn) => (
        <>
          <Show when={turn.asked}>
            {(asked) => (
              <p data-asked>
                {props.text.stage.asked}: {asked()}
              </p>
            )}
          </Show>
          <aside data-clarified>
            <Rich text={turn.answer} words={props.words} copy={props.copy} />
          </aside>
        </>
      )}
    </For>
  );
}
