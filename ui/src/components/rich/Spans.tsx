import { For, Match, Switch } from "solid-js";

import Snip from "./Snip";
import type { Copier, Words } from "./Snip";
import type { Span } from "../../lib/spans";

interface Props {
  spans: Span[];
  words: Words;
  copy?: Copier;
}

function linked(span: Span): { text: string; href: string } | undefined {
  return span.kind === "link" ? span : undefined;
}

export default function Spans(props: Props) {
  return (
    <For each={props.spans}>
      {(span) => (
        <Switch fallback={span.text}>
          <Match when={span.kind === "code"}>
            <Snip text={span.text} words={props.words} copy={props.copy} />
          </Match>
          <Match when={span.kind === "strong"}>
            <strong>{span.text}</strong>
          </Match>
          <Match when={span.kind === "em"}>
            <em>{span.text}</em>
          </Match>
          <Match when={linked(span)}>
            {(link) => <a href={link().href}>{link().text}</a>}
          </Match>
        </Switch>
      )}
    </For>
  );
}
