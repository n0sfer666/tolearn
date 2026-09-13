import { For, Show } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { ClarificationView, StageIn } from "../../ipc";
import type { Transport } from "../../lib/ipc";
import type { Copier, Words } from "../rich/Snip";
import { settle } from "./settle";
import Turns from "./Turns";

interface Props {
  text: Dictionary;
  call: Transport;
  at: StageIn;
  chains: ClarificationView[];
  words: Words;
  copy?: Copier;
  changed: (clarifications: ClarificationView[]) => void;
}

export default function Orphans(props: Props) {
  const removed = (chain: number) =>
    settle(() => props.call("unclarify", { ...props.at, chain }), props.changed, props.text.stage.clarifyFailed);

  return (
    <Show when={props.chains.length > 0}>
      <details data-orphans>
        <summary>{props.text.stage.orphans}</summary>
        <ul>
          <For each={props.chains}>
            {(chain) => (
              <li data-orphan={chain.chain}>
                <blockquote data-excerpt>{chain.excerpt}</blockquote>
                <Turns text={props.text} turns={chain.turns} words={props.words} copy={props.copy} />
                <button type="button" data-unclarify onClick={() => void removed(chain.chain)}>
                  {props.text.stage.unclarify}
                </button>
              </li>
            )}
          </For>
        </ul>
      </details>
    </Show>
  );
}
