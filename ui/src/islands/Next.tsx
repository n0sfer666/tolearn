import { Show, createMemo, createSignal, onMount } from "solid-js";

import Ahead from "../components/generate/Ahead";
import Outcome from "../components/generate/Outcome";
import Progress from "../components/generate/Progress";
import Refusal from "../components/generate/Refusal";
import Variants from "../components/generate/Variants";
import Empty from "../components/reading/Empty";
import type { Dictionary } from "../i18n/ru";
import type { ForkOut, GenerationStateOut, GenerationWork, NodeOut, StageOut, TakeNextOut, VariantView } from "../ipc";
import { generation } from "../lib/generation";
import { quiet, states, steps } from "../lib/ipc";
import type { Listen, Transport, Watch } from "../lib/ipc";
import { BUILDING, FORKING } from "../lib/jobs";
import { landed } from "../lib/launched";
import { go, stageHref } from "../lib/links";
import { ongoing } from "../lib/ongoing";
import { query } from "../lib/query";
import { ranked } from "../lib/ranked";
import { regain } from "../lib/regain";
import { resumable } from "../lib/resumable";
import { slot } from "../lib/slot";
import { leave } from "../lib/worded";

interface Props {
  text: Dictionary;
  locale: string;
  program?: string;
  node?: string;
  stage?: string;
  call?: Transport;
  steps?: Listen;
  watch?: Watch;
  go?: (href: string) => void;
}

const KINDS = new Set(["fork", "next"]);

export default function Next(props: Props) {
  const call = () => props.call ?? quiet;
  const work = generation(props.text, call, props.steps ?? steps);
  const live = ongoing(call, props.watch ?? states);
  const at = () => ({
    program: props.program ?? query("program"),
    node: props.node ?? query("node"),
    stage: props.stage ?? query("stage"),
  });

  const [variants, setVariants] = createSignal<VariantView[]>([]);
  const [opened, setOpened] = createSignal(false);
  const [missing, setMissing] = createSignal(false);
  const [map, setMap] = createSignal<NodeOut | null>(null);
  const [done, setDone] = createSignal<StageOut | null>(null);
  const [hovered, setHovered] = createSignal<number | null>(null);
  const [focused, setFocused] = createSignal(0);
  const [kind, setKind] = createSignal("fork");

  const picks = createMemo(() => ranked(variants()));
  const taken = createMemo(() => slot(map(), at().stage));
  const picked = () => hovered() ?? focused();
  const rows = () => map()?.stages.length ?? 0;
  const place = () => {
    const where = taken();
    if (where === null) return null;
    if (where >= rows()) return props.text.generate.placeOnward;
    return props.text.generate.place.replace("{n}", String(where + 1)).replace("{m}", String(rows()));
  };
  const row = () => map()?.stages.find((stage) => stage.id === at().stage) ?? null;

  const read = async () => {
    const here = at();
    const [node, stage] = await Promise.allSettled([
      call()("node", { program: here.program, node: here.node }),
      call()("stage", here),
    ]);
    setMap(node.status === "fulfilled" ? node.value : null);
    setDone(stage.status === "fulfilled" ? stage.value : null);
  };

  const open = async (draw: () => Promise<ForkOut> = () => call()("fork", at()), from?: GenerationWork) => {
    setOpened(true);
    setHovered(null);
    setFocused(0);
    setKind("fork");
    const found = await work.run(draw, FORKING, from);
    if (found !== null) setVariants(found.variants);
  };

  const build = async (draw: () => Promise<TakeNextOut>, from?: GenerationWork) => {
    setKind("next");
    const built = await work.run(draw, BUILDING, from);
    if (built !== null) (props.go ?? go)(stageHref(props.locale, at().program, built.node, built.stage));
  };

  const choose = (choice: number) => build(() => call()("take_next", { ...at(), choice }));

  const ours = (from: GenerationWork) =>
    from.program === at().program && from.node === at().node && from.stage === at().stage;

  const resume = (state: GenerationStateOut) => {
    const built = state.work === null ? state.outcome : null;
    if (built?.stage && built.work.kind === "next" && ours(built.work)) {
      return void (props.go ?? go)(landed(props.locale, built.stage));
    }
    const from = resumable(state, KINDS, ours);
    if (from === null) return void open();
    if (from.kind === "fork") return void open(() => live.ended(from, (outcome) => outcome.fork), from);
    setOpened(true);
    void build(() => live.ended(from, (outcome) => outcome.stage), from);
  };

  onMount(() => {
    const here = at();
    if (here.program !== "" && here.stage !== "") {
      void read();
      void live.known.then(resume);
      return;
    }
    setMissing(true);
  });

  const idle = () => opened() && !work.running();

  return (
    <Show
      when={!missing()}
      fallback={<Empty reason={props.text.generate.none} locale={props.locale} label={props.text.nav.library} />}
    >
      <div data-fork data-studio>
        <div data-studio-ask>
          <h2>{props.text.generate.variants}</h2>
          <p>{props.text.generate.variantsLead}</p>
          <Show when={work.running()}>
            <Progress text={props.text} work={work} hint={leave(kind(), props.text)} />
          </Show>
          <Show when={idle() && picks().length > 0}>
            <Variants
              text={props.text}
              picks={picks()}
              picked={picked()}
              place={place()}
              focus={regain(work.calm)}
              attend={setFocused}
              hover={setHovered}
              choose={(choice) => void choose(choice)}
            />
          </Show>
          <Show when={idle() && picks().length === 0}>
            <button type="button" data-retry ref={regain(() => work.ended() === "halted")} onClick={() => void open()}>
              {props.text.generate.retry}
            </button>
          </Show>
          <Refusal text={props.text} locale={props.locale} refused={work.refused()} />
        </div>
        <div data-studio-side>
          <Show when={done()}>{(view) => <Outcome text={props.text} view={view()} pass={row()?.pass ?? null} />}</Show>
        </div>
        <Show when={map() !== null && taken() !== null && picks().length > 0}>
          <Ahead
            text={props.text}
            stages={map()?.stages ?? []}
            slot={taken() ?? 0}
            title={picks()[picked()]?.variant.title ?? ""}
          />
        </Show>
      </div>
    </Show>
  );
}
