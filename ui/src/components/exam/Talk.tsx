import { For } from "solid-js";

import Rich from "../rich/Rich";
import type { Copier } from "../rich/Snip";
import type { Dictionary } from "../../i18n/ru";
import type { ExamLineView } from "../../ipc";

interface Props {
  text: Dictionary["dialog"];
  log: ExamLineView[];
  copy?: Copier;
}

export default function Talk(props: Props) {
  const who = (side: string) => (side === "student" ? props.text.student : props.text.examiner);
  const words = () => ({
    copied: props.text.copied,
    manual: props.text.copyManually,
    label: props.text.copyCode,
  });

  return (
    <ol data-talk>
      <For each={props.log}>
        {(line) => (
          <li data-side={line.side}>
            <b>{who(line.side)}</b>
            <Rich text={line.text} words={words()} copy={props.copy} />
          </li>
        )}
      </For>
    </ol>
  );
}
