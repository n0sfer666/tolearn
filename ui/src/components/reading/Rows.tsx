import { For, Show } from "solid-js";

import type { RowView } from "../../ipc";
import { hours } from "../../lib/hours";

interface Props {
  kind: "stage" | "child";
  rows: RowView[];
  href: (row: RowView) => string;
  pending: string;
  unit: string;
}

export default function Rows(props: Props) {
  return (
    <For each={props.rows}>
      {(row) => (
        <li
          data-stage={props.kind === "stage" ? row.id : undefined}
          data-child={props.kind === "child" ? row.id : undefined}
          data-pending={row.ready ? undefined : ""}
        >
          <Show when={row.ready} fallback={<span data-row-title>{row.title}</span>}>
            <a href={props.href(row)}>{row.title}</a>
          </Show>
          <span data-hours>{hours(row.hours, props.unit)}</span>
          <Show when={!row.ready}>
            <span data-note>{props.pending}</span>
          </Show>
        </li>
      )}
    </For>
  );
}
