import { Show } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { HttpView } from "../../ipc";
import { heat, numbered, tenths } from "../../lib/provider";

interface Props {
  text: Dictionary;
  value: HttpView;
  local: boolean;
  onChange: (next: HttpView) => void;
}

const NUM_CTX_MAX = 1048576;

export default function HttpFields(props: Props) {
  return (
    <>
      <label>
        {props.text.provider.endpoint}
        <input
          data-endpoint
          type="url"
          value={props.value.endpoint}
          onInput={(event) =>
            props.onChange({ ...props.value, endpoint: event.currentTarget.value })
          }
        />
      </label>

      <label>
        {props.text.provider.model}
        <input
          data-model
          type="text"
          value={props.value.model}
          onInput={(event) => props.onChange({ ...props.value, model: event.currentTarget.value })}
        />
      </label>

      <Show when={props.value.api === "ollama"}>
        <label>
          {props.text.provider.context}
          <input
            data-num-ctx
            type="number"
            min="0"
            max={NUM_CTX_MAX}
            step="1024"
            value={props.value.num_ctx}
            onInput={(event) =>
              props.onChange({
                ...props.value,
                num_ctx: numbered(event.currentTarget.value, NUM_CTX_MAX),
              })
            }
          />
        </label>
      </Show>

      <Show when={props.local && props.value.api === "openai"}>
        <p data-context-hint>{props.text.provider.contextHint}</p>
      </Show>

      <label>
        {props.text.provider.temperature}
        <input
          data-temperature
          type="number"
          min="0"
          max="2"
          step="0.1"
          value={heat(props.value.temperature_tenths)}
          onInput={(event) =>
            props.onChange({
              ...props.value,
              temperature_tenths: tenths(event.currentTarget.value),
            })
          }
        />
      </label>
    </>
  );
}
