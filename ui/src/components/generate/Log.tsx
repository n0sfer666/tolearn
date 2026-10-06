import type { Dictionary } from "../../i18n/ru";
import type { Log } from "../../lib/log";
import Room from "../journal/Room";

interface Props {
  text: Dictionary;
  log: Log;
}

export default function LogTab(props: Props) {
  return (
    <div data-log>
      <Room text={props.text} log={props.log} />
    </div>
  );
}
