import { For, Show } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { AskView, StageIn } from "../../ipc";
import { drafts } from "../../lib/drafts";
import { generation } from "../../lib/generation";
import type { Listen, Transport } from "../../lib/ipc";
import { EXAMINING } from "../../lib/jobs";
import { regain } from "../../lib/regain";
import { toast } from "../../lib/toast";
import Progress from "../generate/Progress";
import Refusal from "../generate/Refusal";
import type { Copier, Words } from "../rich/Snip";
import Paste from "./Paste";
import Question from "./Question";
import type { Voice } from "./voice";

interface Props {
  text: Dictionary;
  locale: string;
  call: Transport;
  listen: Listen;
  at: StageIn;
  questions: AskView[];
  voice: Voice;
  words: Words;
  copy?: Copier;
  done: () => void;
}

export default function Exam(props: Props) {
  const work = generation(props.text, () => props.call, props.listen);
  const store = drafts(
    (question, text) => props.call("answer", { ...props.at, question, text }),
    () => toast("error", props.text.stage.unsaved),
  );
  const typed = (ask: AskView) => store.typed(ask.id) ?? ask.draft;
  const answers = () => props.questions.map((ask) => ({ id: ask.id, text: typed(ask) }));

  const submit = async () => {
    const given = answers();
    if (given.every((answer) => answer.text.trim() === "")) {
      work.refuse(props.text.stage.blank);
      return;
    }
    const sat = await work.run(async () => {
      await store.flush();
      return props.call("exam", { ...props.at, answers: given });
    }, EXAMINING);
    if (sat === null) return;
    toast(sat.passed ? "ok" : "info", sat.passed ? props.text.stage.passed : props.text.stage.graded);
    props.done();
  };

  return (
    <Show when={props.questions.length > 0}>
      <section data-questions>
        <h3>{props.text.stage.questions}</h3>
        <ol>
          <For each={props.questions}>
            {(ask) => (
              <Question
                ask={ask}
                text={props.text}
                words={props.words}
                copy={props.copy}
                answer={typed(ask)}
                voice={props.voice}
                locked={work.running()}
                onType={(text) => store.type(ask.id, text)}
                onLeave={() => void store.leave(ask.id)}
              />
            )}
          </For>
        </ol>
        <div data-examination>
          <Show when={!work.running()} fallback={<Progress text={props.text} work={work} />}>
            <button type="button" data-exam ref={regain(work.calm)} onClick={() => void submit()}>
              {props.text.stage.submit}
            </button>
          </Show>
          <Refusal text={props.text} locale={props.locale} refused={work.refused()} />
        </div>
        <Paste
          text={props.text}
          locale={props.locale}
          call={props.call}
          at={props.at}
          answers={answers}
          flush={store.flush}
          locked={work.running()}
          copy={props.copy}
          done={props.done}
        />
      </section>
    </Show>
  );
}
