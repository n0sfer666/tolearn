import { For } from "solid-js";

import type { Hints } from "../../i18n/hints/shape";
import type { Dictionary } from "../../i18n/ru";
import type { HarnessView, PresetView } from "../../ipc";
import { argued, preset } from "../../lib/provider";
import Hint from "../Hint";
import ArgsHint from "./ArgsHint";

interface Props {
  text: Dictionary;
  hints: Hints;
  value: HarnessView;
  presets: PresetView[];
  onChange: (next: HarnessView) => void;
}

export default function HarnessFields(props: Props) {
  const chosen = () => props.presets.find((known) => known.id === props.value.id);

  const pick = (id: string) => {
    const known = props.presets.find((one) => one.id === id);
    if (known === undefined) return;
    if (known.command === "") {
      props.onChange({ ...props.value, id, args: [] });
      return;
    }
    props.onChange({ ...props.value, id, command: known.command, args: [] });
  };

  const advisable = () => (chosen()?.args.length ?? 0) > 0;

  const advise = () => {
    const known = chosen();
    if (known === undefined) return;
    props.onChange({ ...props.value, args: [...known.args] });
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
        <span data-args-label>
          {props.text.provider.args}
          <Hint label={props.hints.open}>
            <ArgsHint text={props.text} hints={props.hints} preset={props.value.id} />
          </Hint>
        </span>
        <textarea
          data-args
          rows={4}
          value={props.value.args.join("\n")}
          onInput={(event) => props.onChange({ ...props.value, args: lines(event.currentTarget.value) })}
        />
      </label>
      <button type="button" data-args-advise disabled={!advisable()} onClick={advise}>
        {props.hints.apply}
      </button>
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
