import { For, Show } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { PlanOut } from "../../ipc";
import { grab } from "../../lib/grab";
import { hours } from "../../lib/hours";

interface Props {
  text: Dictionary;
  out: PlanOut;
}

export default function PlanMap(props: Props) {
  const unit = () => props.text.program.hours;

  return (
    <section data-plan-map>
      <h2 tabIndex={-1} ref={grab}>
        {props.out.plan.title}
      </h2>
      <p>{props.out.plan.goal}</p>
      <p>
        {props.text.program.span}: <span data-hours>{hours(props.out.hours, unit())}</span>
      </p>
      <h3>{props.text.program.stages}</h3>
      <ol>
        <For each={props.out.plan.stages}>
          {(stage) => (
            <li data-plan-stage={stage.id}>
              <span>{stage.title}</span>
              <span data-hours>{hours(stage.hours, unit())}</span>
            </li>
          )}
        </For>
      </ol>
      <Show when={props.out.plan.children.length > 0}>
        <h3>{props.text.program.children}</h3>
        <ul>
          <For each={props.out.plan.children}>
            {(part) => (
              <li data-plan-part>
                <strong>{part.title}</strong>
                <span data-hours>{hours(part.hours, unit())}</span>
                <p>{part.goal}</p>
              </li>
            )}
          </For>
        </ul>
      </Show>
    </section>
  );
}
