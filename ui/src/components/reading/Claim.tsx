import { Show } from "solid-js";

import Run from "./Run";
import type { Desk } from "./desk";
import Snip from "../rich/Snip";
import type { Copier, Words } from "../rich/Snip";
import type { Dictionary } from "../../i18n/ru";
import type { ClaimView } from "../../ipc";

interface Props {
  claim: ClaimView;
  desk: Desk;
  text: Dictionary;
  words: Words;
  copy?: Copier;
}

export default function Claim(props: Props) {
  const ticked = () => props.desk.ticks().includes(props.claim.id);

  const flip = async (box: HTMLInputElement) => {
    const on = box.checked;
    if (!(await props.desk.tick(props.claim.id, on))) box.checked = !on;
  };

  return (
    <li id={props.claim.id} data-claim={props.claim.id} data-ticked={ticked() ? "" : undefined}>
      <label>
        <input type="checkbox" data-tick checked={ticked()} onChange={(event) => void flip(event.currentTarget)} />
        <span>{props.claim.claim}</span>
      </label>
      <Show when={props.claim.check}>
        {(check) => (
          <span data-command>
            {props.text.stage.check}: <Snip text={check()} words={props.words} copy={props.copy} />
          </span>
        )}
      </Show>
      <Show when={props.claim.expect !== ""}>
        <span data-expect>
          {props.text.stage.expect}: {props.claim.expect}
        </span>
      </Show>
      <Show when={props.claim.check !== null}>
        <Run claim={props.claim.id} desk={props.desk} text={props.text} />
      </Show>
    </li>
  );
}
