import { Show } from "solid-js";

import type { CheckView, RunCheckOut } from "../../ipc";
import type { Dictionary } from "../../i18n/ru";

export interface Ran {
  out: RunCheckOut | null;
  failed: boolean;
}

interface Props {
  text: Dictionary;
  check: CheckView;
  ran: Ran | undefined;
  running: boolean;
  done: boolean;
  onRun: (id: string) => void;
  onDone: (id: string) => void;
}

export default function CheckItem(props: Props) {
  return (
    <li data-check={props.check.id}>
      <p data-claim>{props.check.claim}</p>
      <code data-command>{props.check.check}</code>
      <p data-expect>
        {props.text.practice.expect}: {props.check.expect}
      </p>
      <button type="button" data-run disabled={props.running} onClick={() => props.onRun(props.check.id)}>
        {props.running ? props.text.practice.running : props.text.practice.run}
      </button>
      <button
        type="button"
        data-done
        aria-pressed={props.done}
        onClick={() => props.onDone(props.check.id)}
      >
        {props.text.practice.done}
      </button>
      <Show when={props.ran?.failed}>
        <p data-failed>{props.text.practice.failed}</p>
      </Show>
      <Show when={props.ran?.out}>
        {(out) => (
          <div data-output>
            <p data-code>
              {props.text.practice.code}: {out().code ?? "—"}
            </p>
            <Show when={out().timed_out}>
              <p data-timeout>{props.text.practice.timedOut}</p>
            </Show>
            <pre>{out().stdout}</pre>
            <Show when={out().stderr}>
              <pre data-stderr>{out().stderr}</pre>
            </Show>
            <Show when={out().truncated}>
              <p data-truncated>{props.text.practice.truncated}</p>
            </Show>
          </div>
        )}
      </Show>
    </li>
  );
}
