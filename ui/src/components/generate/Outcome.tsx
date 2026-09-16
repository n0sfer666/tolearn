import { For, Show } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { AskView, StageOut } from "../../ipc";
import { graded, score } from "../../lib/scored";

interface Props {
  text: Dictionary;
  view: StageOut;
  pass: string | null;
}

export default function Outcome(props: Props) {
  const asked = () => props.view.questions;
  const missed = () => asked().filter((ask) => ask.result !== null && ask.result !== "ok");
  const claims = () => props.view.practice.constraints.length + props.view.practice.acceptance.length;
  const verdict = () => {
    if (props.pass === "skip") return props.text.generate.outcomeSkipped;
    if (graded(asked())) return score(props.text.stage.score, asked());
    return props.text.generate.outcomeUntried;
  };
  const clarified = () => props.text.generate.clarified.replace("{n}", String(props.view.clarifications.length));
  const ticked = () =>
    props.text.generate.ticked
      .replace("{n}", String(props.view.ticks.length))
      .replace("{m}", String(claims()));

  return (
    <section data-outcome>
      <h2>{props.text.generate.outcome}</h2>
      <p data-stage-title>{props.view.title}</p>
      <p data-verdict>{verdict()}</p>
      <Show when={missed().length > 0}>
        <h3>{props.text.generate.missedTitle}</h3>
        <ul data-missed>
          <For each={missed()}>{(ask: AskView) => <li data-missed-question={ask.id}>{ask.text}</li>}</For>
        </ul>
      </Show>
      <p data-clarified>{clarified()}</p>
      <Show when={claims() > 0}>
        <p data-ticked>{ticked()}</p>
      </Show>
    </section>
  );
}
