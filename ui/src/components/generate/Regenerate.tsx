import { Show, onMount } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { GenerationStateOut, GenerationWork, ReadyStage, RegenerateStageIn } from "../../ipc";
import { generation } from "../../lib/generation";
import type { Listen, Transport, Watch } from "../../lib/ipc";
import { BUILDING } from "../../lib/jobs";
import { ongoing } from "../../lib/ongoing";
import { regain } from "../../lib/regain";
import { resumable } from "../../lib/resumable";
import { toast } from "../../lib/toast";
import Progress from "./Progress";
import Refusal from "./Refusal";

interface Props {
  text: Dictionary;
  locale: string;
  call: Transport;
  listen: Listen;
  watch: Watch;
  at: RegenerateStageIn;
  done: () => void;
}

const KINDS = new Set(["regenerate"]);

export default function Regenerate(props: Props) {
  const work = generation(props.text, () => props.call, props.listen);
  const live = ongoing(() => props.call, props.watch);

  const settle = async (draw: () => Promise<unknown>, from?: GenerationWork) => {
    const done = await work.run(draw, BUILDING, from);
    if (done === null) return;
    toast("ok", props.text.generate.regenerated);
    props.done();
  };

  const ours = (from: GenerationWork) =>
    from.program === props.at.program && from.node === props.at.node && from.stage === props.at.stage;

  const resume = (state: GenerationStateOut) => {
    const from = resumable(state, KINDS, ours);
    if (from === null || work.running()) return;
    void settle(() => live.ended<ReadyStage>(from, (outcome) => outcome.stage), from);
  };

  onMount(() => void live.known.then(resume));

  return (
    <div data-regeneration>
      <Show
        when={!work.running()}
        fallback={<Progress text={props.text} work={work} hint={props.text.generate.leaveRegenerate} />}
      >
        <button
          type="button"
          data-regenerate
          data-action
          ref={regain(work.calm)}
          onClick={() => void settle(() => props.call("regenerate_stage", props.at))}
        >
          {props.text.generate.regenerate}
        </button>
      </Show>
      <Refusal text={props.text} locale={props.locale} refused={work.refused()} />
    </div>
  );
}
