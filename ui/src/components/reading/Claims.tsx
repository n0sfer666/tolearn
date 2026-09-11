import { For, Show } from "solid-js";

import Snip from "../rich/Snip";
import type { Copier, Words } from "../rich/Snip";
import type { Dictionary } from "../../i18n/ru";
import type { ClaimView } from "../../ipc";

interface Props {
  title: string;
  claims: ClaimView[];
  text: Dictionary;
  words: Words;
  copy?: Copier;
}

export default function Claims(props: Props) {
  return (
    <Show when={props.claims.length > 0}>
      <h4>{props.title}</h4>
      <ul data-claims>
        <For each={props.claims}>
          {(claim) => (
            <li id={claim.id} data-claim={claim.id}>
              <span>{claim.claim}</span>
              <Show when={claim.check}>
                {(check) => (
                  <span data-command>
                    {props.text.stage.check}: <Snip text={check()} words={props.words} copy={props.copy} />
                  </span>
                )}
              </Show>
              <Show when={claim.expect !== ""}>
                <span data-expect>
                  {props.text.stage.expect}: {claim.expect}
                </span>
              </Show>
            </li>
          )}
        </For>
      </ul>
    </Show>
  );
}
