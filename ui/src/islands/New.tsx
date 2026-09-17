import { Show, createEffect, createSignal } from "solid-js";

import Intent from "../components/generate/Intent";
import LogTab from "../components/generate/Log";
import PlanMap from "../components/generate/PlanMap";
import PlanTools from "../components/generate/PlanTools";
import Progress from "../components/generate/Progress";
import Refusal from "../components/generate/Refusal";
import Steps from "../components/generate/Steps";
import Tabs from "../components/generate/Tabs";
import type { Dictionary } from "../i18n/ru";
import type { PlanOut, StartProgramOut } from "../ipc";
import { generation } from "../lib/generation";
import { quiet, steps } from "../lib/ipc";
import type { Listen, Transport } from "../lib/ipc";
import { BUILDING, PLANNING, REVISING } from "../lib/jobs";
import type { Job } from "../lib/jobs";
import { go, nodeHref, stageHref } from "../lib/links";
import { log } from "../lib/log";
import { marks } from "../lib/marks";
import { regain } from "../lib/regain";

interface Props {
  text: Dictionary;
  locale: string;
  call?: Transport;
  steps?: Listen;
  go?: (href: string) => void;
}

const MAP = "map";
const LOG = "log";
const TABS = [MAP, LOG];

interface Asked {
  request: string;
  level: string;
  locale: string;
}

interface Drawn {
  want: Asked;
  out: PlanOut;
}

function opened(locale: string, done: StartProgramOut): string {
  if (done.stage === "") return nodeHref(locale, done.program);
  return stageHref(locale, done.program, done.node, done.stage);
}

function track(text: Dictionary) {
  return [
    { key: "asked", label: text.generate.markAsked },
    { key: "reach", label: text.generate.markReach },
    { key: "drawn", label: text.generate.markDrawn },
  ];
}

export default function New(props: Props) {
  const call = () => props.call ?? quiet;
  const work = generation(props.text, call, props.steps ?? steps);
  const clock = marks();
  const kept = log(props.text, call);

  const [request, setRequest] = createSignal("");
  const [level, setLevel] = createSignal("");
  const [tongue, setTongue] = createSignal(props.locale);
  const [wish, setWish] = createSignal("");
  const [drawn, setDrawn] = createSignal<Drawn | null>(null);
  const [tab, setTab] = createSignal(MAP);
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

  const traced = async (job: Job, draw: () => Promise<PlanOut>): Promise<PlanOut | null> => {
    clock.start(track(props.text));
    clock.note("asked");
    const done = await work.run(draw, job);
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
    const done = await traced(REVISING, () => call()("revise_plan", { ...want, plan: shown.out.plan, wish: change }));
    if (done === null) return;
    setDrawn({ want, out: done });
    setWish("");
  };

  const start = async (shown: Drawn) => {
    pressed = "start";
    clock.start([]);
    const done = await work.run(() => call()("start_program", { ...shown.want, plan: shown.out.plan }), BUILDING);
    kept.ask(false, false);
    if (done !== null) (props.go ?? go)(opened(props.locale, done));
  };

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
        <Show when={!work.running()} fallback={<Progress text={props.text} work={work} />}>
          <Show when={drawn() === null}>{begin()}</Show>
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
