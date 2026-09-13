import type { Dictionary } from "../../i18n/ru";
import type { Generation } from "../../lib/generation";
import { grab } from "../../lib/grab";
import { stepped } from "../../lib/stepped";

interface Props {
  text: Dictionary;
  work: Generation;
}

export default function Progress(props: Props) {
  const said = () =>
    props.work.cancelling() ? props.text.generate.cancelling : stepped(props.work.step(), props.text);

  return (
    <div data-progress role="status">
      <p data-step>{said()}</p>
      <button
        type="button"
        data-cancel
        ref={grab}
        aria-disabled={props.work.cancelling() ? "true" : undefined}
        onClick={() => props.work.cancel()}
      >
        {props.text.generate.cancel}
      </button>
    </div>
  );
}
