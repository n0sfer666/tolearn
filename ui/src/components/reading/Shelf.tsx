import { Show } from "solid-js";

import Rows from "./Rows";
import type { Dictionary } from "../../i18n/ru";
import type { ShelfView } from "../../ipc";
import { hours } from "../../lib/hours";
import { nodeHref } from "../../lib/links";

interface Props {
  shelf: ShelfView;
  text: Dictionary["programs"];
  locale: string;
}

export default function Shelf(props: Props) {
  return (
    <li data-program={props.shelf.uuid}>
      <a href={nodeHref(props.locale, props.shelf.uuid)}>{props.shelf.title}</a>
      <p data-goal>{props.shelf.goal}</p>
      <p data-hours>{hours(props.shelf.hours, props.text.hours)}</p>
      <Show when={props.shelf.children.length > 0}>
        <ul data-children aria-label={props.text.children}>
          <Rows
            kind="child"
            rows={props.shelf.children}
            href={(row) => nodeHref(props.locale, props.shelf.uuid, row.id)}
            pending={props.text.pending}
            unit={props.text.hours}
          />
        </ul>
      </Show>
    </li>
  );
}
