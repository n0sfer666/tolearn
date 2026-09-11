import { Match, Switch } from "solid-js";

import Picture from "./Picture";
import Rich from "../rich/Rich";
import Snip from "../rich/Snip";
import type { Copier, Words } from "../rich/Snip";
import type { Dictionary } from "../../i18n/ru";
import type { BlockView } from "../../ipc";

interface Props {
  block: BlockView;
  text: Dictionary;
  words: Words;
  copy?: Copier;
}

const PICTURES = new Set(["diagram", "image"]);

export default function Block(props: Props) {
  const said = () => <Rich text={props.block.text} words={props.words} copy={props.copy} />;

  return (
    <Switch
      fallback={
        <div id={props.block.id} data-block={props.block.kind}>
          {said()}
        </div>
      }
    >
      <Match when={props.block.kind === "heading"}>
        <h3 id={props.block.id} data-block="heading">
          {props.block.text}
        </h3>
      </Match>
      <Match when={props.block.kind === "callout"}>
        <aside id={props.block.id} data-block="callout">
          {said()}
        </aside>
      </Match>
      <Match when={props.block.kind === "code"}>
        <pre id={props.block.id} data-block="code" data-lang={props.block.lang ?? undefined} data-rich-code>
          <Snip text={props.block.text} words={props.words} copy={props.copy} />
        </pre>
      </Match>
      <Match when={PICTURES.has(props.block.kind)}>
        <Picture block={props.block} text={props.text} />
      </Match>
    </Switch>
  );
}
