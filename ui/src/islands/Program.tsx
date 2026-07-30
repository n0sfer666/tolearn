import { For, Show, createSignal, onMount } from "solid-js";

import { GLYPHS, type Status } from "../components/status";
import type { Dictionary } from "../i18n/ru";
import { type Locale, plural } from "../i18n";
import type { Stage, TopicStatus } from "../ipc";
import { matches } from "../lib/filter";
import { pickFile, transport } from "../lib/ipc";
import type { Transport } from "../lib/ipc";

interface Props {
  text: Dictionary;
  locale: Locale;
  path?: string;
  today?: string;
  call?: Transport;
  save?: (name: string) => Promise<string | null>;
}

function known(status: string): status is Status {
  return status in GLYPHS;
}

function glyph(status: string): string {
  return known(status) ? GLYPHS[status] : GLYPHS.todo;
}

export default function Program(props: Props) {
  const call = () => props.call ?? transport;
  const today = () => props.today ?? new Date().toISOString().slice(0, 10);
  const path = () => props.path ?? new URLSearchParams(location.search).get("program") ?? "";

  const [stages, setStages] = createSignal<Stage[]>([]);
  const [topics, setTopics] = createSignal<TopicStatus[]>([]);
  const [needle, setNeedle] = createSignal("");
  const [program, setProgram] = createSignal("");
  const [state, setState] = createSignal<"idle" | "busy" | "done" | "failed">("idle");

  const [gone, setGone] = createSignal(false);

  onMount(() => {
    setProgram(path());
    if (path() === "") return;
    void (async () => {
      try {
        const out = await call()("program", { bundle: path(), today: today() });
        setStages(out.stages);
        setTopics(out.topics);
      } catch {
        setGone(true);
      }
    })();
  });

  const name = () => `${path().split(/[/\\]/).filter(Boolean).at(-1) ?? "program"}.md`;

  const exported = async () => {
    const chosen = await (props.save ?? pickFile)(name());
    if (chosen === null) return;
    setState("busy");
    try {
      await call()("export", { bundle: path(), today: today(), path: chosen, directory: null });
      setState("done");
    } catch {
      setState("failed");
    }
  };

  const of = (stage: Stage) =>
    topics().filter((topic) => topic.stage === stage.n && matches(topic.title, needle()));
  const label = (status: string) => (known(status) ? props.text.status[status] : status);
  const href = (topic: TopicStatus) =>
    `/${props.locale}/topic/?program=${encodeURIComponent(path())}&topic=${encodeURIComponent(topic.id)}`;

  return (
    <Show
      when={program() !== "" && !gone()}
      fallback={
        <p data-empty>
          {props.text.program.none}{" "}
          <a href={`/${props.locale}/`}>{props.text.nav.programs}</a>
        </p>
      }
    >
      <section>
        <nav class="row" data-tools aria-label={props.text.nav.sections}>
          <a data-stale href={`/${props.locale}/stale/?program=${encodeURIComponent(program())}`}>
            {props.text.stale.title}
          </a>
          <a data-stats href={`/${props.locale}/stats/?program=${encodeURIComponent(program())}`}>
            {props.text.stats.title}
          </a>
          <a data-graph href={`/${props.locale}/graph/?program=${encodeURIComponent(program())}`}>
            {props.text.graph.title}
          </a>
          <button type="button" data-export onClick={() => void exported()} disabled={state() === "busy"}>
            {state() === "busy" ? props.text.program.exporting : props.text.program.export}
          </button>
        </nav>
        <Show when={state() === "done" || state() === "failed"}>
          <p data-exported role="status">
            {state() === "done" ? props.text.program.exported : props.text.program.exportFailed}
          </p>
        </Show>
        <input
          type="search"
          data-filter
          aria-label={props.text.program.filter}
          placeholder={props.text.program.filter}
          value={needle()}
          onInput={(event) => setNeedle(event.currentTarget.value)}
        />
        <For each={stages()}>
          {(stage) => (
            <article data-stage={stage.n}>
              <div class="head">
                <h3>{stage.title}</h3>
                <p data-tally>
                  {props.text.programs.progress}: {stage.tally.done} /{" "}
                  {plural(props.locale, stage.tally.total, props.text.counts.topics)}
                </p>
              </div>
              <ul>
                <For each={of(stage)}>
                  {(topic) => (
                    <li data-topic={topic.id} data-checkpoint={topic.checkpoint ? "" : undefined}>
                      <span class={`glyph status-${topic.status}`} role="img" aria-label={label(topic.status)}>
                        {glyph(topic.status)}
                      </span>
                      <Show when={topic.blocked_by.length === 0} fallback={<span>{topic.title}</span>}>
                        <a href={href(topic)}>{topic.title}</a>
                      </Show>
                      <span data-status>{label(topic.status)}</span>
                      <Show when={topic.checkpoint}>
                        <span data-kind>{props.text.program.checkpoint}</span>
                      </Show>
                      <span data-hours>
                        {topic.hours.min}–{topic.hours.max} {props.text.program.hours}
                      </span>
                      <Show when={topic.blocked_by.length > 0}>
                        <p data-blocked>
                          {props.text.program.blockedBy}:{" "}
                          {topic.blocked_by.map((link) => link.title).join(", ")}
                        </p>
                      </Show>
                    </li>
                  )}
                </For>
              </ul>
            </article>
          )}
        </For>
      </section>
    </Show>
  );
}
