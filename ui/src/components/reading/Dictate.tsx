import { Show, onCleanup } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import { spliced } from "../../lib/dictation";
import type { Voice } from "./voice";

interface Props {
  text: Dictionary;
  voice: Voice;
  field: string;
  area: () => HTMLTextAreaElement | undefined;
  locked: boolean;
  put: (text: string) => void;
}

export default function Dictate(props: Props) {
  const phase = () => {
    const hold = props.voice.held();
    return hold?.field === props.field ? hold.phase : null;
  };
  const listening = () => phase() === "listening";
  const free = () => props.voice.held() === null;

  const label = () => {
    if (phase() === "listening") return props.text.stage.hush;
    if (phase() === "hearing") return props.text.stage.hearing;
    return props.text.stage.dictate;
  };

  const insert = (heard: string) => {
    const area = props.area();
    if (area === undefined) return;
    const next = spliced(area.value, area.selectionStart, area.selectionEnd, heard);
    props.put(next.text);
    area.focus();
    area.setSelectionRange(next.caret, next.caret);
  };

  onCleanup(() => props.voice.drop(props.field));

  return (
    <Show when={props.voice.available()}>
      <button
        type="button"
        data-dictate
        data-listening={listening()}
        aria-pressed={listening()}
        disabled={props.locked || !(free() || listening())}
        onClick={() => void props.voice.toggle(props.field, insert)}
      >
        {label()}
      </button>
    </Show>
  );
}
