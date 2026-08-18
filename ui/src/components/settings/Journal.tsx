import { Show, createSignal, onMount } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { LlmLogOut } from "../../ipc";
import type { Transport } from "../../lib/ipc";
import { reason } from "../../lib/provider";
import { toast } from "../../lib/toast";

interface Props {
  text: Dictionary;
  call: Transport;
  on: boolean;
  onToggle: (next: boolean) => void;
}

export default function Journal(props: Props) {
  const [log, setLog] = createSignal<LlmLogOut | null>(null);

  const ask = (open: boolean, clear: boolean) => {
    void (async () => {
      try {
        setLog(await props.call("llm_log", { open, clear }));
      } catch (error) {
        toast("error", reason(error, props.text));
      }
    })();
  };

  onMount(() => ask(false, false));

  return (
    <>
      <label>
        <input
          data-journal
          type="checkbox"
          checked={props.on}
          onChange={(event) => props.onToggle(event.currentTarget.checked)}
        />
        {props.text.provider.journal}
      </label>
      <p data-journal-lead>{props.text.provider.journalLead}</p>
      <Show when={log()}>
        {(kept) => (
          <>
            <p data-journal-room>
              {props.text.provider.journalKept} {kept().records} · {kept().room}
            </p>
            <button type="button" data-journal-open onClick={() => ask(true, false)}>
              {props.text.provider.journalOpen}
            </button>
            <button type="button" data-journal-clear onClick={() => ask(false, true)}>
              {props.text.provider.journalClear}
            </button>
          </>
        )}
      </Show>
    </>
  );
}
