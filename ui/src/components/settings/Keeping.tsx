import type { Dictionary } from "../../i18n/ru";

interface Props {
  text: Dictionary;
  on: boolean;
  onToggle: (next: boolean) => void;
}

export default function Keeping(props: Props) {
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
    </>
  );
}
