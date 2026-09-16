import type { Dictionary } from "../../i18n/ru";
import type { PlanView } from "../../ipc";

interface Props {
  text: Dictionary;
  plan: PlanView;
  wish: string;
  back: (button: string) => (element: HTMLElement) => void;
  setWish: (value: string) => void;
  revise: () => void;
  start: () => void;
}

export default function PlanTools(props: Props) {
  const first = () => props.plan.stages[0];
  const begin = () => {
    const stage = first();
    if (stage === undefined) return props.text.generate.start;
    return props.text.generate.startFirst.replace("{stage}", stage.title);
  };

  return (
    <div data-plan-tools>
      <label>
        {props.text.generate.wish}
        <textarea
          data-wish
          rows="2"
          value={props.wish}
          onInput={(event) => props.setWish(event.currentTarget.value)}
        />
      </label>
      <button type="button" data-revise ref={props.back("revise")} onClick={() => props.revise()}>
        {props.text.generate.revise}
      </button>
      <button type="button" data-start ref={props.back("start")} onClick={() => props.start()}>
        {begin()}
      </button>
    </div>
  );
}
