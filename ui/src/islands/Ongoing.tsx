import { Show, createEffect } from "solid-js";

import type { Dictionary } from "../i18n/ru";
import type { GenerationOutcome } from "../ipc";
import { clocked } from "../lib/clocked";
import { elapsed } from "../lib/elapsed";
import { generation } from "../lib/generation";
import { here } from "../lib/here";
import { quiet, states, steps } from "../lib/ipc";
import type { Listen, Transport, Watch } from "../lib/ipc";
import { jobOf } from "../lib/jobs";
import { launched, opened } from "../lib/launched";
import { ongoing } from "../lib/ongoing";
import { ready } from "../lib/worded";

interface Props {
  text: Dictionary;
  locale: string;
  folded?: boolean;
  call?: Transport;
  steps?: Listen;
  watch?: Watch;
}

const MINUTE = 60_000;

export default function Ongoing(props: Props) {
  const call = () => props.call ?? quiet;
  const live = ongoing(call, props.watch ?? states);
  const work = generation(props.text, call, props.steps ?? steps, false);
  const minutes = elapsed(work.began, MINUTE);
  let followed: number | null = null;
  let acked: number | null = null;

  const current = () => live.state()?.work ?? null;
  const shown = (outcome: GenerationOutcome) => here(opened(props.locale, outcome));
  const finished = () => {
    const state = live.state();
    if (state === null || state.work !== null || state.outcome === null || state.outcome.seen) return null;
    return state.outcome;
  };
  const notice = () => {
    const outcome = finished();
    return outcome === null || shown(outcome) ? null : outcome;
  };
  const folded = () =>
    props.text.generate.ongoingFolded.replace("{n}", String(Math.floor(minutes() / MINUTE)));

  createEffect(() => {
    const now = current();
    if (now === null || now.began === followed) return;
    followed = now.began;
    void work.run(() => live.ended(now, (outcome) => outcome), jobOf(now.kind), now);
  });

  createEffect(() => {
    const outcome = finished();
    if (outcome === null || outcome.stage === null || outcome.work.began === acked) return;
    if (!shown(outcome)) return;
    acked = outcome.work.began;
    live.seen();
  });

  return (
    <>
      <Show when={current()}>
        {(now) => (
          <div data-ongoing role="group" aria-label={props.text.generate.ongoing}>
            <a
              href={launched(props.locale, now())}
              data-ongoing-link
              aria-label={`${props.text.generate.ongoing}: ${work.said()}`}
            >
              <Show
                when={props.folded}
                fallback={
                  <>
                    <span data-step>{work.said()}</span>
                    <span data-elapsed>{clocked(work.spent(), props.text)}</span>
                  </>
                }
              >
                {folded()}
              </Show>
            </a>
            <button
              type="button"
              data-ongoing-cancel
              data-action
              aria-label={props.text.generate.cancelGeneration}
              aria-disabled={work.cancelling() ? "true" : undefined}
              onClick={() => work.cancel()}
            >
              {props.text.generate.cancel}
            </button>
          </div>
        )}
      </Show>
      <Show when={notice()}>
        {(outcome) => (
          <p data-ongoing-done>
            <a href={opened(props.locale, outcome())} data-ongoing-open>
              {ready(outcome(), props.text)}
            </a>
            <button
              type="button"
              data-ongoing-close
              aria-label={props.text.generate.dismiss}
              onClick={() => live.seen()}
            >
              ×
            </button>
          </p>
        )}
      </Show>
    </>
  );
}
