import { For } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { ExamLineView } from "../../ipc";

interface Props {
  text: Dictionary["dialog"];
  log: ExamLineView[];
}

export default function Talk(props: Props) {
  const who = (side: string) => (side === "student" ? props.text.student : props.text.examiner);

  return (
    <ol data-talk>
      <For each={props.log}>
        {(line) => (
          <li data-side={line.side}>
            <b>{who(line.side)}</b>
            <p>{line.text}</p>
          </li>
        )}
      </For>
    </ol>
  );
}
