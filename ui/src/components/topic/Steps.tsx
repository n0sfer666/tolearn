import { Show } from "solid-js";

import { moves, type Step } from "./moves";
import type { Status } from "../status";
import type { Dictionary } from "../../i18n/ru";

interface Props {
  text: Dictionary;
  status: string;
  onPick: (status: Status) => void;
}

interface MoveProps {
  text: Dictionary;
  way: "back" | "next";
  step: Step;
  onPick: (status: Status) => void;
}

function Move(props: MoveProps) {
  return (
    <button
      type="button"
      data-step={props.way}
      data-status-choice={props.step.to}
      onClick={() => props.onPick(props.step.to)}
    >
      {props.text.steps[props.step.word]}
    </button>
  );
}

export default function Steps(props: Props) {
  const move = () => moves(props.status);
  const stuck = () => move().back === undefined && move().next === undefined;

  return (
    <p data-steps>
      <Show when={move().back}>
        {(step) => <Move text={props.text} way="back" step={step()} onPick={props.onPick} />}
      </Show>
      <Show when={move().next}>
        {(step) => <Move text={props.text} way="next" step={step()} onPick={props.onPick} />}
      </Show>
      <Show when={stuck()}>
        <span data-steps-locked>{props.text.steps.locked}</span>
      </Show>
    </p>
  );
}
