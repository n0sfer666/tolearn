import { Show } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { CheckClaimOut } from "../../ipc";

interface Props {
  out: CheckClaimOut;
  text: Dictionary;
}

export default function Output(props: Props) {
  const stage = () => props.text.stage;
  const ended = () => {
    const code = props.out.code;
    return code === null ? stage().killed : stage().code.replace("{code}", String(code));
  };
  const limit = () => {
    if (props.out.outcome === "timeout") return stage().timeout;
    if (props.out.outcome === "silence") return stage().silence;
    return stage().stopped;
  };
  const silent = () => props.out.stdout === "" && props.out.stderr === "";

  return (
    <div data-output data-outcome={props.out.outcome}>
      <Show when={props.out.outcome === "finished"} fallback={<span data-limit>{limit()}</span>}>
        <span data-code>{ended()}</span>
      </Show>
      <Show when={props.out.truncated}>
        <span data-truncated>{stage().truncated}</span>
      </Show>
      <Show when={props.out.stdout !== ""}>
        <pre data-stdout>{props.out.stdout}</pre>
      </Show>
      <Show when={props.out.stderr !== ""}>
        <pre data-stderr>{props.out.stderr}</pre>
      </Show>
      <Show when={silent()}>
        <span data-silent>{stage().quiet}</span>
      </Show>
    </div>
  );
}
