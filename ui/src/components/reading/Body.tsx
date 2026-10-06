import { For, Show, createSignal } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { BlockView, ClarificationView, StageIn } from "../../ipc";
import type { Listen, Transport } from "../../lib/ipc";
import type { Aim } from "../../lib/picked";
import type { Copier, Words } from "../rich/Snip";
import Block from "./Block";
import Clarify from "./Clarify";
import Pick from "./Pick";
import type { Voice } from "./voice";

interface Props {
  text: Dictionary;
  locale: string;
  call: Transport;
  listen: Listen;
  at: StageIn;
  blocks: BlockView[];
  chains: ClarificationView[];
  voice: Voice;
  words: Words;
  copy?: Copier;
  changed: (clarifications: ClarificationView[]) => void;
}

export default function Body(props: Props) {
  const [aim, setAim] = createSignal<Aim | null>(null);
  const [body, setBody] = createSignal<HTMLElement>();

  return (
    <div data-stage-body ref={setBody}>
      <For each={props.blocks}>
        {(block) => (
          <>
            <Block block={block} text={props.text} words={props.words} copy={props.copy} />
            <Show when={block.kind !== "heading"}>
              <Clarify
                text={props.text}
                locale={props.locale}
                call={props.call}
                listen={props.listen}
                at={props.at}
                block={block}
                chains={props.chains.filter((chain) => chain.block === block.id)}
                aim={aim()}
                voice={props.voice}
                words={props.words}
                copy={props.copy}
                close={() => setAim(null)}
                changed={props.changed}
              />
            </Show>
          </>
        )}
      </For>
      <Pick text={props.text} body={body} pick={(aimed) => setAim(aimed)} />
    </div>
  );
}
