import { Show } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { StageRowView } from "../../ipc";

interface Props {
  text: Dictionary["program"];
  row: StageRowView;
}

export default function Mark(props: Props) {
  const status = () =>
    new Map([
      ["fresh", props.text.fresh],
      ["opened", props.text.opened],
      ["passed", props.text.passed],
    ]).get(props.row.status) ?? props.row.status;
  const pass = () =>
    new Map([
      ["exam", props.text.exam],
      ["skip", props.text.skipped],
    ]).get(props.row.pass ?? "");

  return (
    <>
      <span data-status={props.row.status}>{status()}</span>
      <Show when={pass()}>{(label) => <span data-pass={props.row.pass ?? ""}>{label()}</span>}</Show>
    </>
  );
}
