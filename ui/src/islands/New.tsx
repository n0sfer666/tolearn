import { Show, createEffect, createSignal, onMount } from "solid-js";

import Intent from "../components/generate/Intent";
import LogTab from "../components/generate/Log";
import PlanMap from "../components/generate/PlanMap";
import PlanTools from "../components/generate/PlanTools";
import Progress from "../components/generate/Progress";
import Refusal from "../components/generate/Refusal";
import Steps from "../components/generate/Steps";
import Tabs from "../components/generate/Tabs";
import type { Dictionary } from "../i18n/ru";
import type { GenerationStateOut, GenerationWork, PlanOut, ReadyStage } from "../ipc";
import { stale } from "../lib/drawn";
import type { Asked, Drawn } from "../lib/drawn";
import { generation } from "../lib/generation";
import { quiet, states, steps } from "../lib/ipc";
import type { Listen, Transport, Watch } from "../lib/ipc";
import { BUILDING, PLANNING, REVISING, jobOf } from "../lib/jobs";
import type { Job } from "../lib/jobs";
import { landed } from "../lib/launched";
import { go } from "../lib/links";
import { log } from "../lib/log";
import { marks } from "../lib/marks";
import { ongoing } from "../lib/ongoing";
import { planned } from "../lib/planned";
import { regain } from "../lib/regain";
import { resumable } from "../lib/resumable";
import { track } from "../lib/track";
import { leave } from "../lib/worded";

interface Props {
  text: Dictionary;
  locale: string;
  call?: Transport;
  steps?: Listen;
  watch?: Watch;
  go?: (href: string) => void;
}

const MAP = "map";
const LOG = "log";
const TABS = [MAP, LOG];
const KINDS = new Set(["plan", "revise", "start"]);

export default function New(props: Props) {
  const call = () => props.call ?? quiet;
  const work = generation(props.text, call, props.steps ?? steps);
  const live = ongoing(call, props.watch ?? states);
  const clock = marks();
  const kept = log(props.text, call);

  const [request, setRequest] = createSignal("");
  const [level, setLevel] = createSignal("");
  const [tongue, setTongue] = createSignal(props.locale);
  const [wish, setWish] = createSignal("");
  const [drawn, setDrawn] = createSignal<Drawn | null>(null);
  const [tab, setTab] = createSignal(MAP);
  const [kind, setKind] = createSignal("plan");
  let pressed = "";

  const asked = (): Asked => ({ request: request().trim(), level: level().trim(), locale: tongue() });
  const back = (button: string) => regain(() => work.ended() === "halted" && pressed === button);
  const named = (key: string) => {
    if (key === MAP) return props.text.generate.map;
    return props.text.generate.log.replace("{n}", String(kept.records()));
  };

  createEffect(() => {
    if (work.step() !== null) clock.hit("reach");
  });

  const traced = async (job: Job, draw: () => Promise<PlanOut>, from?: GenerationWork): Promise<PlanOut | null> => {
    clock.start(track(props.text), from?.began);
    clock.note("asked");
    const reach = from?.marks[0]?.at;
    if (reach !== undefined) clock.hit("reach", reach);
    const done = await work.run(draw, job, from);
    kept.ask(false, false);
    if (done === null) return null;
    clock.note("reach");
    clock.hit("drawn");
    return done;
  };

  const plan = async () => {
    const want = asked();
    if (want.request === "" || want.level === "") {
      work.refuse(props.text.generate.missing);
      return;
    }
    pressed = "plan";
    setKind("plan");
    const done = await traced(PLANNING, () => call()("plan_program", want));
    if (done !== null) setDrawn({ want, out: done });
  };

  const revise = async (shown: Drawn) => {
    const change = wish().trim();
    if (change === "") {
      work.refuse(props.text.generate.wishMissing);
      return;
    }
    const want = asked();
    pressed = "revise";
    setKind("revise");
    const done = await traced(REVISING, () => call()("revise_plan", { ...want, plan: shown.out.plan, wish: change }));
    if (done === null) return;
    setDrawn({ want, out: done });
    setWish("");
  };

  const build = async (draw: () => Promise<ReadyStage>, from?: GenerationWork) => {
    pressed = "start";
    setKind("start");
    clock.start([]);
    const done = await work.run(draw, BUILDING, from);
    kept.ask(false, false);
    if (done !== null) (props.go ?? go)(landed(props.locale, done));
  };

  const start = (shown: Drawn) => build(() => call()("start_program", { ...shown.want, plan: shown.out.plan }));

  const redraw = async (from: GenerationWork, running: boolean) => {
    pressed = from.kind;
    setKind(from.kind);
    const draw = () => live.ended(from, (outcome) => outcome.plan);
    const job = jobOf(from.kind);
    const done = running ? await traced(job, draw, from) : await work.run(draw, job, from);
    if (done === null) return;
    setDrawn({ want: asked(), out: done });
    setWish("");
  };

  const restore = (state: GenerationStateOut) => {
    const from = resumable(state, KINDS);
    if (from === null || work.running()) return;
    setRequest(from.request);
    setLevel(from.level);
    setTongue(from.locale);
    setWish(from.wish);
    if (from.plan !== null) setDrawn({ want: asked(), out: planned(from.plan) });
    if (from.kind === "start") void build(() => live.ended(from, (outcome) => outcome.stage), from);
    else void redraw(from, state.work !== null);
  };

  onMount(() => void live.known.then(restore));

  const begin = () => (
    <button type="button" data-plan ref={back("plan")} onClick={() => void plan()}>
      {props.text.generate.plan}
    </button>
  );

  return (
    <div data-new data-studio>
      <div data-studio-ask>
        <Intent
          text={props.text}
          request={request()}
          level={level()}
          locale={tongue()}
          locked={work.running()}
          setRequest={setRequest}
          setLevel={setLevel}
          setLocale={setTongue}
        />
        <Show when={!work.running()} fallback={<Progress text={props.text} work={work} hint={leave(kind(), props.text)} />}>
          <Show when={stale(asked(), drawn())}>{begin()}</Show>
        </Show>
        <Refusal text={props.text} locale={props.locale} refused={work.refused()} />
      </div>
      <div data-studio-side>
        <Steps text={props.text} seen={clock.seen} work={work} />
        <Tabs keys={TABS} label={named} current={tab()} pick={setTab} />
        <Show when={tab() === MAP} fallback={<LogTab text={props.text} log={kept} />}>
          <Show when={drawn()} keyed>
            {(shown) => (
              <>
                <PlanMap text={props.text} out={shown.out} />
                <Show when={!work.running()}>
                  <PlanTools
                    text={props.text}
                    plan={shown.out.plan}
                    wish={wish()}
                    back={back}
                    setWish={setWish}
                    revise={() => void revise(shown)}
                    start={() => void start(shown)}
                  />
                </Show>
              </>
            )}
          </Show>
        </Show>
      </div>
    </div>
  );
}
