import { Show } from "solid-js";

import { type Locale, plural } from "../../i18n";
import type { Dictionary } from "../../i18n/ru";
import type { ShelfView } from "../../ipc";
import { ago } from "../../lib/ago";
import { day } from "../../lib/day";
import { hours } from "../../lib/hours";
import { nodeHref } from "../../lib/links";
import { counted, summary } from "../../lib/summary";

interface Props {
  shelf: ShelfView;
  text: Dictionary["programs"];
  locale: Locale;
  today?: Date;
}

function size(props: Props): string {
  const { stages, subprograms } = props.shelf;
  if (subprograms > 0) return plural(props.locale, subprograms, props.text.subprograms);
  return plural(props.locale, stages, props.text.stages);
}

export default function Shelf(props: Props) {
  return (
    <li data-program={props.shelf.uuid}>
      <div data-about>
        <a href={nodeHref(props.locale, props.shelf.uuid)}>{props.shelf.title}</a>
        <p data-goal>{props.shelf.goal}</p>
        <p data-meta>
          <span data-size>{size(props)}</span>
          <span data-hours>{hours(props.shelf.hours, props.text.hours)}</span>
          <Show when={props.shelf.active}>
            {(on) => (
              <time data-active datetime={on()} title={`${props.text.active}: ${day(on(), props.locale)}`}>
                {ago(on(), props.locale, props.today ?? new Date())}
              </time>
            )}
          </Show>
        </p>
      </div>
      <div data-passed>
        <Show when={counted(props.shelf.summary)}>
          {(view) => (
            <>
              <progress value={view().passed} max={view().total} aria-label={props.text.progress} />
              <p data-summary>{summary(props.text.summary, view())}</p>
            </>
          )}
        </Show>
        <Show when={props.shelf.unread}>
          {(reason) => (
            <p data-unread>
              {props.text.unread}: {reason()}
            </p>
          )}
        </Show>
      </div>
    </li>
  );
}
