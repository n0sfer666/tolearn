import { For } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { HarnessView, PresetView } from "../../ipc";
import { argued, preset } from "../../lib/provider";

interface Props {
  text: Dictionary;
  value: HarnessView;
  presets: PresetView[];
  onChange: (next: HarnessView) => void;
}

export default function HarnessFields(props: Props) {
  const pick = (id: string) => {
    const chosen = props.presets.find((known) => known.id === id);
    if (chosen === undefined) return;
    if (chosen.command === "") {
      props.onChange({ ...props.value, id });
      return;
    }
    props.onChange({ ...props.value, id, command: chosen.command, args: chosen.args });
  };

  const timeout = (raw: string) => {
    const seconds = Number.parseInt(raw, 10);
    if (Number.isNaN(seconds)) return;
    props.onChange({ ...props.value, timeout_secs: seconds });
  };

  return (
    <>
      <label>
        {props.text.provider.preset}
        <select data-preset value={props.value.id} onChange={(event) => pick(event.currentTarget.value)}>
          <For each={props.presets}>
            {(known) => <option value={known.id}>{preset(known.id, props.text)}</option>}
          </For>
        </select>
      </label>

      <label>
        {props.text.provider.command}
        <input
          data-command
          type="text"
          value={props.value.command}
          onInput={(event) => props.onChange({ ...props.value, command: event.currentTarget.value })}
        />
      </label>

      <label>
        {props.text.provider.args}
        <textarea
          data-args
          rows={4}
          value={props.value.args.join("\n")}
          onInput={(event) => props.onChange({ ...props.value, args: lines(event.currentTarget.value) })}
        />
      </label>
      <p data-args-seen>{argued(props.value.args, props.text)}</p>
      <p data-args-warning>{props.text.provider.argsWarning}</p>

      <label>
        {props.text.provider.timeout}
        <input
          data-timeout
          type="number"
          min="1"
          max="3600"
          value={props.value.timeout_secs}
          onInput={(event) => timeout(event.currentTarget.value)}
        />
      </label>
    </>
  );
}

function lines(text: string): string[] {
  return text === "" ? [] : text.split("\n");
}
