import { Show } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { ShelfView } from "../../ipc";
import { day } from "../../lib/day";
import { hours } from "../../lib/hours";
import { nodeHref } from "../../lib/links";
import { counted, summary } from "../../lib/summary";

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
      <Show when={counted(props.shelf.summary)}>
        {(view) => (
          <>
            <progress value={view().passed} max={view().total} aria-label={props.text.progress} />
            <p data-summary>{summary(props.text.summary, view())}</p>
          </>
        )}
      </Show>
      <Show when={props.shelf.active}>
        {(on) => (
          <p data-active>
            {props.text.active}: {day(on(), props.locale)}
          </p>
        )}
      </Show>
      <Show when={props.shelf.unread}>
        {(reason) => (
          <p data-unread>
            {props.text.unread}: {reason()}
          </p>
        )}
      </Show>
    </li>
  );
}
