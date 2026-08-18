import { Show, createSignal, onCleanup, onMount } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { PracticeOut } from "../../ipc";
import type { Transport } from "../../lib/ipc";

interface Props {
  text: Dictionary;
  program: string;
  topic: string;
  call: Transport;
}

function clocked(seconds: number): string {
  const whole = Math.floor(Math.abs(seconds));
  const minutes = Math.floor(whole / 60);
  const rest = whole % 60;
  return `${String(minutes).padStart(2, "0")}:${String(rest).padStart(2, "0")}`;
}

export default function Timer(props: Props) {
  const [taken, setTaken] = createSignal<PracticeOut | null>(null);
  const [anchor, setAnchor] = createSignal(0);
  const [ticked, setTicked] = createSignal(0);

  const step = (name: string) => {
    void (async () => {
      const out = await props.call("practice", {
        bundle: props.program,
        topic: props.topic,
        step: name,
        now: new Date().toISOString(),
      });
      setAnchor(Date.now());
      setTicked(Date.now());
      setTaken(out);
    })();
  };

  onMount(() => {
    step("peek");
    const beat = setInterval(() => setTicked(Date.now()), 1000);
    onCleanup(() => clearInterval(beat));
  });

  const drift = (out: PracticeOut) =>
    out.running ? Math.floor((ticked() - anchor()) / 1000) : 0;
  const left = (out: PracticeOut) => out.left_sec - drift(out);
  const spent = (out: PracticeOut) => out.spent_sec + drift(out);

  return (
    <Show when={taken()}>
      {(out) => (
        <section data-clock>
          <p data-timer data-over={left(out()) < 0 ? "" : undefined}>
            {left(out()) < 0 ? "−" : ""}
            {clocked(left(out()))}
          </p>
          <p data-spent>
            {props.text.practice.spent}: {clocked(spent(out()))} / {out().box_min}{" "}
            {props.text.topic.minutes}
          </p>
          <Show when={out().expired}>
            <p data-expired>{props.text.practice.over}</p>
          </Show>
          <Show
            when={out().running}
            fallback={
              <button type="button" data-start onClick={() => step("start")}>
                {props.text.practice.start}
              </button>
            }
          >
            <button type="button" data-pause onClick={() => step("pause")}>
              {props.text.practice.pause}
            </button>
          </Show>
          <button type="button" data-reset onClick={() => step("reset")}>
            {props.text.practice.reset}
          </button>
        </section>
      )}
    </Show>
  );
}
