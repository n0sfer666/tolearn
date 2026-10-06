import Room from "../components/journal/Room";
import type { Dictionary } from "../i18n/ru";
import { quiet } from "../lib/ipc";
import type { Transport } from "../lib/ipc";
import { log } from "../lib/log";

interface Props {
  text: Dictionary;
  call?: Transport;
}

export default function Ledger(props: Props) {
  const kept = log(props.text, () => props.call ?? quiet);

  return (
    <section data-card="journal">
      <h2>{props.text.journal.title}</h2>
      <Room text={props.text} log={kept} />
    </section>
  );
}
