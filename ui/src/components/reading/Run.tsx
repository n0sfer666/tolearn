import { Show, createSignal } from "solid-js";

import Output from "./Output";
import type { Desk } from "./desk";
import type { Dictionary } from "../../i18n/ru";
import type { CheckClaimOut } from "../../ipc";

interface Props {
  claim: string;
  desk: Desk;
  text: Dictionary;
}

export default function Run(props: Props) {
  const [ran, setRan] = createSignal<CheckClaimOut | null>(null);
  const [failed, setFailed] = createSignal("");
  const [busy, setBusy] = createSignal(false);

  const unset = () => props.desk.workdir() === null;

  const check = async () => {
    setFailed("");
    try {
      setRan(await props.desk.check(props.claim));
    } catch (failure) {
      setRan(null);
      setFailed(props.desk.reason(failure));
    }
  };

  const press = async () => {
    if (busy()) return;
    setBusy(true);
    await (unset() ? props.desk.choose() : check());
    setBusy(false);
  };

  const label = () => {
    if (busy()) return props.text.stage.running;
    return unset() ? props.text.stage.choose : props.text.stage.run;
  };

  return (
    <>
      <button type="button" data-run onClick={() => void press()} aria-disabled={busy()} aria-busy={busy()}>
        {label()}
      </button>
      <Show when={failed() !== ""}>
        <p data-failed role="alert">
          {failed()}
        </p>
      </Show>
      <Show when={ran()}>{(out) => <Output out={out()} text={props.text} />}</Show>
    </>
  );
}
