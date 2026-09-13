import { For, Show } from "solid-js";
import type { JSX } from "solid-js";

import type { Span } from "../../ipc";
import { hours } from "../../lib/hours";

interface Row {
  id: string;
  title: string;
  hours: Span;
  ready: boolean;
}

interface Props<TRow extends Row> {
  kind: "stage" | "child";
  rows: TRow[];
  href: (row: TRow) => string;
  pending: string;
  unit: string;
  mark?: (row: TRow) => JSX.Element;
}

export default function Rows<TRow extends Row>(props: Props<TRow>) {
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
          {props.mark?.(row)}
          <Show when={!row.ready}>
            <span data-note>{props.pending}</span>
          </Show>
        </li>
      )}
    </For>
  );
}
