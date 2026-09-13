import { Show, createSignal } from "solid-js";

import Intent from "../components/generate/Intent";
import PlanMap from "../components/generate/PlanMap";
import Progress from "../components/generate/Progress";
import Refusal from "../components/generate/Refusal";
import type { Dictionary } from "../i18n/ru";
import type { PlanOut, StartProgramOut } from "../ipc";
import { generation } from "../lib/generation";
import { quiet, steps } from "../lib/ipc";
import type { Listen, Transport } from "../lib/ipc";
import { BUILDING, PLANNING, REVISING } from "../lib/jobs";
import { go, nodeHref, stageHref } from "../lib/links";
import { regain } from "../lib/regain";

interface Props {
  text: Dictionary;
  locale: string;
  call?: Transport;
  steps?: Listen;
  go?: (href: string) => void;
}

function opened(locale: string, done: StartProgramOut): string {
  if (done.stage === "") return nodeHref(locale, done.program);
  return stageHref(locale, done.program, done.node, done.stage);
}

export default function New(props: Props) {
  const call = () => props.call ?? quiet;
  const work = generation(props.text, call, props.steps ?? steps);

  const [request, setRequest] = createSignal("");
  const [level, setLevel] = createSignal("");
  const [wish, setWish] = createSignal("");
  const [out, setOut] = createSignal<PlanOut | null>(null);
  let pressed = "";

  const asked = () => ({ request: request().trim(), level: level().trim() });
  const back = (button: string) => regain(() => work.ended() === "halted" && pressed === button);

  const plan = async () => {
    const { request: want, level: know } = asked();
    if (want === "" || know === "") {
      work.refuse(props.text.generate.missing);
      return;
    }
    pressed = "plan";
    const done = await work.run(() => call()("plan_program", { request: want, level: know }), PLANNING);
    if (done !== null) setOut(done);
  };

  const revise = async (shown: PlanOut) => {
    const change = wish().trim();
    if (change === "") {
      work.refuse(props.text.generate.wishMissing);
      return;
    }
    pressed = "revise";
    const done = await work.run(() => call()("revise_plan", { ...asked(), plan: shown.plan, wish: change }), REVISING);
    if (done === null) return;
    setOut(done);
    setWish("");
  };

  const start = async (shown: PlanOut) => {
    pressed = "start";
    const done = await work.run(() => call()("start_program", { ...asked(), plan: shown.plan }), BUILDING);
    if (done !== null) (props.go ?? go)(opened(props.locale, done));
  };

  const ask = (
    <Intent
      text={props.text}
      request={request()}
      level={level()}
      locked={work.running()}
      setRequest={setRequest}
      setLevel={setLevel}
    />
  );

  const begin = () => (
    <button type="button" data-plan ref={back("plan")} onClick={() => void plan()}>
      {props.text.generate.plan}
    </button>
  );

  return (
    <div data-new>
      <Show when={out()} keyed fallback={ask}>
        {(shown) => <PlanMap text={props.text} out={shown} />}
      </Show>
      <Show when={!work.running()} fallback={<Progress text={props.text} work={work} />}>
        <Show when={out()} fallback={begin()}>
          {(shown) => (
            <div data-plan-tools>
              <label>
                {props.text.generate.wish}
                <textarea data-wish rows="2" value={wish()} onInput={(event) => setWish(event.currentTarget.value)} />
              </label>
              <button type="button" data-revise ref={back("revise")} onClick={() => void revise(shown())}>
                {props.text.generate.revise}
              </button>
              <button type="button" data-start ref={back("start")} onClick={() => void start(shown())}>
                {props.text.generate.start}
              </button>
            </div>
          )}
        </Show>
      </Show>
      <Refusal text={props.text} locale={props.locale} refused={work.refused()} />
    </div>
  );
}
