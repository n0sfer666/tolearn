import type { Dictionary } from "../../i18n/ru";
import type { Transport } from "../../lib/ipc";
import { log } from "../../lib/log";
import Room from "../journal/Room";

interface Props {
  text: Dictionary;
  call: Transport;
  on: boolean;
  onToggle: (next: boolean) => void;
}

export default function Journal(props: Props) {
  const kept = log(props.text, () => props.call);

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
      <Room text={props.text} log={kept} />
    </>
  );
}
