import { For, Show } from "solid-js";

import { GLYPHS, STATUSES, type Status } from "../status";
import type { Dictionary } from "../../i18n/ru";
import type { TopicOut } from "../../ipc";

interface Props {
  text: Dictionary;
  topic: TopicOut;
  onPick: (status: Status) => void;
}

function known(status: string): status is Status {
  return status in GLYPHS;
}

export default function Header(props: Props) {
  const label = (status: string) => (known(status) ? props.text.status[status] : status);
  const glyph = (status: string) => (known(status) ? GLYPHS[status] : GLYPHS.todo);

  return (
    <header data-header>
      <h2>{props.topic.title}</h2>
      <p>
        <span
          class={`glyph status-${props.topic.status}`}
          role="img"
          aria-label={label(props.topic.status)}
        >
          {glyph(props.topic.status)}
        </span>
        <span data-status>{label(props.topic.status)}</span>
        <span data-hours>
          {props.topic.hours.min}–{props.topic.hours.max} {props.text.program.hours}
        </span>
        <Show when={props.topic.outdated}>
          <span data-outdated>{props.text.topic.outdated}</span>
        </Show>
      </p>
      <Show when={props.topic.blocked_by.length > 0}>
        <p data-blocked>
          {props.text.program.blockedBy}:{" "}
          {props.topic.blocked_by.map((link) => link.title).join(", ")}
        </p>
      </Show>
      <p data-mark>
        <span>{props.text.topic.mark}</span>
        <For each={STATUSES}>
          {(status) => (
            <button
              type="button"
              data-status-choice={status}
              aria-pressed={props.topic.status === status}
              onClick={() => props.onPick(status)}
            >
              {props.text.status[status]}
            </button>
          )}
        </For>
      </p>
    </header>
  );
}
