import { Show } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import { RELEASES } from "../../lib/releases";

interface Props {
  text: Dictionary["dialog"];
  available: boolean;
  listening: boolean;
  busy: boolean;
  onToggle: () => void;
}

export default function Voice(props: Props) {
  return (
    <>
      <button
        type="button"
        data-voice
        data-listening={props.listening}
        disabled={!props.available || props.busy}
        onClick={props.onToggle}
      >
        {props.listening ? props.text.hush : props.text.voice}
      </button>
      <Show when={!props.available}>
        <span data-voice-off>
          {props.text.voiceOff}{" "}
          <a href={RELEASES} data-voice-variant>
            {props.text.variant}
          </a>
        </span>
      </Show>
    </>
  );
}
