import { Show } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { RegenerateStageIn } from "../../ipc";
import { generation } from "../../lib/generation";
import type { Listen, Transport } from "../../lib/ipc";
import { BUILDING } from "../../lib/jobs";
import { regain } from "../../lib/regain";
import { toast } from "../../lib/toast";
import Progress from "./Progress";
import Refusal from "./Refusal";

interface Props {
  text: Dictionary;
  locale: string;
  call: Transport;
  listen: Listen;
  at: RegenerateStageIn;
  done: () => void;
}

export default function Regenerate(props: Props) {
  const work = generation(props.text, () => props.call, props.listen);

  const again = async () => {
    const done = await work.run(() => props.call("regenerate_stage", props.at), BUILDING);
    if (done === null) return;
    toast("ok", props.text.generate.regenerated);
    props.done();
  };

  return (
    <div data-regeneration>
      <Show when={!work.running()} fallback={<Progress text={props.text} work={work} />}>
        <button type="button" data-regenerate ref={regain(work.calm)} onClick={() => void again()}>
          {props.text.generate.regenerate}
        </button>
      </Show>
      <Refusal text={props.text} locale={props.locale} refused={work.refused()} />
    </div>
  );
}
