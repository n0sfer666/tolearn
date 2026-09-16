import { For, Show } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { StageRowView } from "../../ipc";
import { hours } from "../../lib/hours";
import Mark from "../reading/Mark";

interface Props {
  text: Dictionary;
  stages: StageRowView[];
  slot: number;
  title: string;
}

export default function Ahead(props: Props) {
  const onward = () => props.slot >= props.stages.length;
  const rows = (): (StageRowView | null)[] => {
    if (onward()) return props.stages;
    return props.stages.map((row, index) => (index === props.slot ? null : row));
  };

  return (
    <section data-ahead aria-live="polite">
      <h2>{props.text.generate.ahead}</h2>
      <ol data-route>
        <For each={rows()}>
          {(row) => (
            <li
              data-stage={row?.id}
              data-slot={row === null ? "" : undefined}
              data-pending={row !== null && !row.ready ? "" : undefined}
            >
              <span data-row-title>{row === null ? props.title : row.title}</span>
              <Show when={row}>
                {(kept) => (
                  <>
                    <span data-hours>{hours(kept().hours, props.text.program.hours)}</span>
                    <Mark text={props.text.program} row={kept()} />
                  </>
                )}
              </Show>
            </li>
          )}
        </For>
      </ol>
      <Show when={onward()}>
        <p data-onward>
          <span data-row-title>{props.title}</span>
          <span data-place>{props.text.generate.placeOnward}</span>
        </p>
      </Show>
    </section>
  );
}
