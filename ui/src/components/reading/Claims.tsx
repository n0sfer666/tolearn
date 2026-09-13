import { For, Show } from "solid-js";

import Claim from "./Claim";
import type { Desk } from "./desk";
import type { Copier, Words } from "../rich/Snip";
import type { Dictionary } from "../../i18n/ru";
import type { ClaimView } from "../../ipc";

interface Props {
  title: string;
  claims: ClaimView[];
  desk: Desk;
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
            <Claim claim={claim} desk={props.desk} text={props.text} words={props.words} copy={props.copy} />
          )}
        </For>
      </ul>
    </Show>
  );
}
