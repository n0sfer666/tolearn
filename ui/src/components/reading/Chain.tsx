import { For, Show, createSignal } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { ClarificationView, ClarificationsOut, StageIn } from "../../ipc";
import type { Transport } from "../../lib/ipc";
import { toast } from "../../lib/toast";
import Rich from "../rich/Rich";
import type { Copier, Words } from "../rich/Snip";
import Asking from "./Asking";

interface Props {
  text: Dictionary;
  call: Transport;
  at: StageIn;
  chain: ClarificationView;
  words: Words;
  copy?: Copier;
  locked: boolean;
  ask: (chain: number, question: string) => Promise<boolean>;
  changed: (clarifications: ClarificationView[]) => void;
}

export default function Chain(props: Props) {
  const [asking, setAsking] = createSignal(false);
  const place = () => ({ ...props.at, chain: props.chain.chain });

  const settle = async (work: () => Promise<ClarificationsOut>) => {
    try {
      props.changed((await work()).clarifications);
    } catch {
      toast("error", props.text.stage.clarifyFailed);
    }
  };

  const followed = async (question: string) => {
    if (await props.ask(props.chain.chain, question)) setAsking(false);
  };

  return (
    <details data-chain open={!props.chain.clear}>
      <summary>{props.text.stage.clarified}</summary>
      <For each={props.chain.turns}>
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
                onClick={() => void settle(() => props.call("understood", place()))}
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
        onClick={() => void settle(() => props.call("unclarify", place()))}
      >
        {props.text.stage.unclarify}
      </button>
    </details>
  );
}
