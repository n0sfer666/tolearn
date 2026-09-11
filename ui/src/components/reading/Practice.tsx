import { For } from "solid-js";

import Block from "./Block";
import Claims from "./Claims";
import Rich from "../rich/Rich";
import type { Copier, Words } from "../rich/Snip";
import type { Dictionary } from "../../i18n/ru";
import type { TaskView } from "../../ipc";

interface Props {
  practice: TaskView;
  text: Dictionary;
  words: Words;
  copy?: Copier;
}

export default function Practice(props: Props) {
  return (
    <section data-practice>
      <h3>{props.text.stage.practice}</h3>
      <p data-later>{props.text.stage.later}</p>
      <For each={props.practice.task}>
        {(block) => <Block block={block} text={props.text} words={props.words} copy={props.copy} />}
      </For>
      <h4>{props.text.stage.deliverable}</h4>
      <div data-deliverable>
        <Rich text={props.practice.deliverable} words={props.words} copy={props.copy} />
      </div>
      <Claims
        title={props.text.stage.constraints}
        claims={props.practice.constraints}
        text={props.text}
        words={props.words}
        copy={props.copy}
      />
      <Claims
        title={props.text.stage.acceptance}
        claims={props.practice.acceptance}
        text={props.text}
        words={props.words}
        copy={props.copy}
      />
    </section>
  );
}
