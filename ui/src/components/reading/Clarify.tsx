import { For, Show, createSignal } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { BlockView, ClarificationView, StageIn } from "../../ipc";
import { generation } from "../../lib/generation";
import type { Listen, Transport } from "../../lib/ipc";
import { CLARIFYING } from "../../lib/jobs";
import Progress from "../generate/Progress";
import Refusal from "../generate/Refusal";
import type { Copier, Words } from "../rich/Snip";
import Asking from "./Asking";
import Chain from "./Chain";
import type { Voice } from "./voice";

interface Props {
  text: Dictionary;
  locale: string;
  call: Transport;
  listen: Listen;
  at: StageIn;
  block: BlockView;
  chains: ClarificationView[];
  voice: Voice;
  words: Words;
  copy?: Copier;
  changed: (clarifications: ClarificationView[]) => void;
}

export default function Clarify(props: Props) {
  const work = generation(props.text, () => props.call, props.listen);
  const [asking, setAsking] = createSignal(false);

  const ask = async (chain: number | null, question: string): Promise<boolean> => {
    const out = await work.run(
      () => props.call("clarify", { ...props.at, block: props.block.id, question, chain }),
      CLARIFYING,
    );
    if (out === null) return false;
    props.changed(out.clarifications);
    return true;
  };

  const opened = async (question: string) => {
    if (await ask(null, question)) setAsking(false);
  };

  return (
    <div data-clarify={props.block.id}>
      <For each={props.chains}>
        {(chain) => (
          <Chain
            text={props.text}
            call={props.call}
            at={props.at}
            chain={chain}
            voice={props.voice}
            words={props.words}
            copy={props.copy}
            locked={work.running()}
            ask={ask}
            changed={props.changed}
          />
        )}
      </For>
      <Show
        when={asking()}
        fallback={
          <button type="button" data-clarify-open disabled={work.running()} onClick={() => setAsking(true)}>
            {props.text.stage.clarify}
          </button>
        }
      >
        <Asking
          text={props.text}
          voice={props.voice}
          field={`block:${props.block.id}`}
          locked={work.running()}
          send={(question) => void opened(question)}
          cancel={() => setAsking(false)}
        />
      </Show>
      <Show when={work.running()}>
        <Progress text={props.text} work={work} />
      </Show>
      <Refusal text={props.text} locale={props.locale} refused={work.refused()} />
    </div>
  );
}
