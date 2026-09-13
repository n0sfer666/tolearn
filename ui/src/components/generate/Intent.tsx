import type { Dictionary } from "../../i18n/ru";

interface Props {
  text: Dictionary;
  request: string;
  level: string;
  locked: boolean;
  setRequest: (value: string) => void;
  setLevel: (value: string) => void;
}

export default function Intent(props: Props) {
  return (
    <div data-intent>
      <label>
        {props.text.generate.request}
        <textarea
          data-request
          rows="3"
          required
          readOnly={props.locked}
          value={props.request}
          onInput={(event) => props.setRequest(event.currentTarget.value)}
        />
      </label>
      <label>
        {props.text.generate.level}
        <textarea
          data-level
          rows="2"
          required
          readOnly={props.locked}
          value={props.level}
          onInput={(event) => props.setLevel(event.currentTarget.value)}
        />
      </label>
    </div>
  );
}
