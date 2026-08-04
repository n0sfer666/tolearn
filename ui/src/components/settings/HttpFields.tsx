import type { Dictionary } from "../../i18n/ru";
import type { HttpView } from "../../ipc";

interface Props {
  text: Dictionary;
  value: HttpView;
  onChange: (next: HttpView) => void;
}

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
    </>
  );
}
