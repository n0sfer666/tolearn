import { For, Show, createSignal, onMount } from "solid-js";

import { GLYPHS, type Status } from "../components/status";
import type { Dictionary } from "../i18n/ru";
import { type Locale, plural } from "../i18n";
import type { Stage, TopicStatus } from "../ipc";
import { matches } from "../lib/filter";
import { transport } from "../lib/ipc";
import type { Transport } from "../lib/ipc";

interface Props {
  text: Dictionary;
  locale: Locale;
  path?: string;
  today?: string;
  call?: Transport;
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

  onMount(() => {
    setProgram(path());
    void (async () => {
      const out = await call()("program", { bundle: path(), today: today() });
      setStages(out.stages);
      setTopics(out.topics);
    })();
  });

  const of = (stage: Stage) =>
    topics().filter((topic) => topic.stage === stage.n && matches(topic.title, needle()));
  const label = (status: string) => (known(status) ? props.text.status[status] : status);
  const href = (topic: TopicStatus) =>
    `/${props.locale}/topic/?program=${encodeURIComponent(path())}&topic=${encodeURIComponent(topic.id)}`;

  return (
    <section>
      <Show when={program() !== ""}>
        <a data-stale href={`/${props.locale}/stale/?program=${encodeURIComponent(program())}`}>
          {props.text.stale.title}
        </a>
        <a data-stats href={`/${props.locale}/stats/?program=${encodeURIComponent(program())}`}>
          {props.text.stats.title}
        </a>
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
            <h3>{stage.title}</h3>
            <p data-tally>
              {props.text.programs.progress}: {stage.tally.done} /{" "}
              {plural(props.locale, stage.tally.total, props.text.counts.topics)}
            </p>
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
  );
}
