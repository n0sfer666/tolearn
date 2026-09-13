import { Show, createSignal } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { ClarificationView, ClarificationsOut, StageIn } from "../../ipc";
import type { Transport } from "../../lib/ipc";
import type { Copier, Words } from "../rich/Snip";
import Asking from "./Asking";
import { settle } from "./settle";
import Turns from "./Turns";
import type { Voice } from "./voice";

interface Props {
  text: Dictionary;
  call: Transport;
  at: StageIn;
  chain: ClarificationView;
  voice: Voice;
  words: Words;
  copy?: Copier;
  locked: boolean;
  ask: (chain: number, question: string) => Promise<boolean>;
  changed: (clarifications: ClarificationView[]) => void;
}

export default function Chain(props: Props) {
  const [asking, setAsking] = createSignal(false);
  const place = () => ({ ...props.at, chain: props.chain.chain });

  const settled = (work: () => Promise<ClarificationsOut>) =>
    settle(work, props.changed, props.text.stage.clarifyFailed);

  const followed = async (question: string) => {
    if (await props.ask(props.chain.chain, question)) setAsking(false);
  };

  return (
    <details data-chain open={!props.chain.clear}>
      <summary>{props.text.stage.clarified}</summary>
      <Turns text={props.text} turns={props.chain.turns} words={props.words} copy={props.copy} />
      <Show when={!props.chain.clear}>
        <Show
          when={asking()}
          fallback={
            <p data-clear>
              {props.text.stage.clear}
              <button
                type="button"
                data-yes
                disabled={props.locked}
                onClick={() => void settled(() => props.call("understood", place()))}
              >
                {props.text.stage.yes}
              </button>
              <button type="button" data-no disabled={props.locked} onClick={() => setAsking(true)}>
                {props.text.stage.no}
              </button>
            </p>
          }
        >
          <Asking
            text={props.text}
            voice={props.voice}
            field={`chain:${props.chain.chain}`}
            locked={props.locked}
            send={(question) => void followed(question)}
            cancel={() => setAsking(false)}
          />
        </Show>
      </Show>
      <button
        type="button"
        data-unclarify
        disabled={props.locked}
        onClick={() => void settled(() => props.call("unclarify", place()))}
      >
        {props.text.stage.unclarify}
      </button>
    </details>
  );
}
