import type { Dictionary } from "../../i18n/ru";
import type { DraftView } from "../../ipc";

interface Props {
  text: Dictionary["generate"];
  draft: DraftView;
  onTake: () => void;
  onDrop: () => void;
}

export default function Resume(props: Props) {
  return (
    <div data-generate-draft>
      <p data-generate-draft-line>
        {props.text.resume} «{props.draft.title}» ({props.draft.done} {props.text.of}{" "}
        {props.draft.total})
      </p>
      <button type="button" data-generate-take onClick={props.onTake}>
        {props.text.go}
      </button>
      <button type="button" data-generate-drop onClick={props.onDrop}>
        {props.text.drop}
      </button>
    </div>
  );
}
