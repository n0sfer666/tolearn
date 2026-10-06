import { For, Show } from "solid-js";

import Block from "./Block";
import Claims from "./Claims";
import type { Desk } from "./desk";
import Workdir from "./Workdir";
import Rich from "../rich/Rich";
import type { Copier, Words } from "../rich/Snip";
import type { Dictionary } from "../../i18n/ru";
import type { TaskView } from "../../ipc";

interface Props {
  practice: TaskView;
  desk: Desk;
  text: Dictionary;
  words: Words;
  copy?: Copier;
}

export default function Practice(props: Props) {
  const runnable = () =>
    [...props.practice.constraints, ...props.practice.acceptance].some((claim) => claim.check !== null);

  return (
    <section data-practice>
      <h3>{props.text.stage.practice}</h3>
      <For each={props.practice.task}>
        {(block) => <Block block={block} text={props.text} words={props.words} copy={props.copy} />}
      </For>
      <h4>{props.text.stage.deliverable}</h4>
      <div data-deliverable>
        <Rich text={props.practice.deliverable} words={props.words} copy={props.copy} />
      </div>
      <Show when={runnable()}>
        <Workdir desk={props.desk} text={props.text} />
      </Show>
      <Claims
        title={props.text.stage.constraints}
        claims={props.practice.constraints}
        desk={props.desk}
        text={props.text}
        words={props.words}
        copy={props.copy}
      />
      <Claims
        title={props.text.stage.acceptance}
        claims={props.practice.acceptance}
        desk={props.desk}
        text={props.text}
        words={props.words}
        copy={props.copy}
      />
    </section>
  );
}
