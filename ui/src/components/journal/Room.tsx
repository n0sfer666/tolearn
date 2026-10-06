import { Show } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { Log } from "../../lib/log";

interface Props {
  text: Dictionary;
  log: Log;
}

export default function Room(props: Props) {
  return (
    <Show when={props.log.kept()}>
      {(kept) => (
        <>
          <p data-journal-room>
            {props.text.journal.kept} {kept().records} · {kept().room}
          </p>
          <button type="button" data-journal-open onClick={() => props.log.ask(true, false)}>
            {props.text.journal.open}
          </button>
          <button type="button" data-journal-clear onClick={() => props.log.ask(false, true)}>
            {props.text.journal.clear}
          </button>
        </>
      )}
    </Show>
  );
}
