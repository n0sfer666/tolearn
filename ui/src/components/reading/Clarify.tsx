import { For, Show } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { BlockView, ClarificationView, StageIn } from "../../ipc";
import { generation } from "../../lib/generation";
import type { Listen, Transport } from "../../lib/ipc";
import { CLARIFYING } from "../../lib/jobs";
import type { Aim } from "../../lib/picked";
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
  aim: Aim | null;
  voice: Voice;
  words: Words;
  copy?: Copier;
  close: () => void;
  changed: (clarifications: ClarificationView[]) => void;
}

export default function Clarify(props: Props) {
  const work = generation(props.text, () => props.call, props.listen);
  const aimed = () => (props.aim?.block === props.block.id ? props.aim : null);

  const ask = async (chain: number | null, question: string, fragment: string | null): Promise<boolean> => {
    const out = await work.run(
      () => props.call("clarify", { ...props.at, block: props.block.id, question, chain, fragment }),
      CLARIFYING,
    );
    if (out === null) return false;
    props.changed(out.clarifications);
    return true;
  };

  const opened = async (fragment: string, question: string) => {
    if (await ask(null, question, fragment === "" ? null : fragment)) props.close();
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
            ask={(chain, question) => ask(chain, question, null)}
            changed={props.changed}
          />
        )}
      </For>
      <Show when={aimed()}>
        {(pick) => (
          <Asking
            text={props.text}
            voice={props.voice}
            field={`block:${props.block.id}`}
            fragment={pick().fragment}
            locked={work.running()}
            send={(question) => void opened(pick().fragment, question)}
            cancel={props.close}
          />
        )}
      </Show>
      <Show when={work.running()}>
        <Progress text={props.text} work={work} />
      </Show>
      <Refusal text={props.text} locale={props.locale} refused={work.refused()} />
    </div>
  );
}
