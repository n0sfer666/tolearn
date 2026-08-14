# Exam protocol

[Русский](../ru/protocol.md) · **English**

The app's contract. This document describes what `tolearn` does with a verdict.

Why it lives in the repository and not in the bundle: `examiner.md` inside a
bundle is a file that the LLM regenerates and the user may edit. It describes what
the **examining model** does. What the **app** does is decided by the app —
otherwise the tracker's behaviour would depend on the contents of an imported
folder. A disagreement between this document and a bundle's `examiner.md` is
resolved in favour of this document; the app shows a warning on import when the
tail of `examiner.md` describes different rules.

## The prompt boundary

`examiner.md` consists of two parts separated by a horizontal rule `---`:

- **before `---`** — the prompt for the examining model. Only this is copied;
- **after `---`** — the "What the tracker does with the result" and "Known
  limitation" sections. Those address the tracker and the person and never reach
  the prompt.

The header block at the top of the file (the "copied into the bundle as
examiner.md" mark) is cut out too: it addresses the generator.

## The verdict

The model's answer ends with exactly one JSON block. The app takes the **last**
block from the pasted text — the whole answer is pasted, and demanding that a
person dig the block out by hand guarantees mistakes.

```json
{
  "topic_id": "...",
  "date": "YYYY-MM-DD",
  "model": "...",
  "verdict": "pass|partial|fail|blocked",
  "hinted": false,
  "practice_accepted": true,
  "failed_checks": [],
  "per_question": [
    {"id": "q1", "result": "ok|partial|miss", "quote": "...",
     "missed": [], "signal_extension": false}
  ],
  "gaps": ["..."],
  "calibration": "...",
  "notes": ["..."],
  "next_action": "proceed|retry_failed|redo_practice|split_topic",
  "retry_after_days": 3
}
```

Required: `topic_id`, `verdict`, `per_question`. A verdict without `per_question`
is rejected — without a per-question breakdown it is no different from eyeballing
a grade. A `topic_id` that does not match the open topic is rejected with an
explicit message: that is almost always an answer pasted into the wrong topic.

With other fields missing, parsing is partial: whatever is there gets applied, and
the list of what was missing is shown.

## Verdict → status

| Verdict | Status | Also |
|---|---|---|
| `pass` | `passed` | `passed_at` = the date; `next_review_at` per `retention` |
| `partial` | `in_progress` | only questions with `result != ok` go into the queue; retry after `retry_after_days` |
| `fail` | `failed` | `gaps[]` highlighted on the analysis screen, practice is redone |
| `blocked` | `in_progress` | **the attempt is not recorded in statistics** |

`blocked` means the exam did not take place: the required artefact was not
produced. That is not a result, so it counts neither towards the attempt counter
nor towards three failures in a row.

## `next_review_at` per `retention`

`N` is the topic's effective revalidation period: its own `revalidate_after_days`,
or `defaults.revalidate_after_days[volatility]` from the header when the field is
absent. The date is computed by adding `N` calendar days to the base, and the base
itself is not counted: `2026-05-01 + 30` is `2026-05-31`.

| `retention` | Base | `next_review_at` | Why that base |
|---|---|---|---|
| `by_schedule` | `passed_at` | `passed_at + N` | it is human memory that fades, so count from the exam |
| `by_use` | the topic's `verified_at` | `verified_at + N` | knowledge is held by practice; it is the subject that ages, not the memory |
| `none` | — | `null` | the topic does not come back |

The base for `by_use` does not depend on the attempt, so the date is set for
`passed_out` as well: the topic was passed by calibration, but the subject ages by
`verified_at` all the same. For `by_use` the date coincides with the shelf life of
the knowledge — the very one `stale_passed` is derived from: a topic enters the
review queue exactly when its content stops being fresh.

## Three failures in a row

Three `fail` verdicts in a row on one topic is a signal that the topic is too
large or that a prerequisite was not met. The app offers to split the topic and
hands over ready-made request text for regeneration in `refresh` mode. It does not
change the bundle itself.

## Marking by hand

Any status can be set by hand, without an exam: a person may have known the topic
already, covered it at work, or disagreed with the model. A manual mark is
recorded with `source: manual` and is distinguished from a passed exam in the
statistics — otherwise the "passed" number stops meaning anything.

The reverse holds too: resetting a status by hand does not erase the attempt
history.

## The shape of an attempt

An `attempts[]` element in `progress.yaml` stores the verdict verbatim plus what
the app knows and the model does not:

```yaml
attempts:
  - at: "2026-07-26T18:40:00+03:00"
    source: exam            # exam | manual
    verdict: partial
    model: "..."
    hinted: false
    practice_accepted: true
    failed_checks: []
    per_question: [...]
    gaps: [...]
    calibration: "..."
    notes: [...]
    next_action: retry_failed
    retry_after_days: 3
    raw: |                  # the original JSON block as it arrived
      {...}
```

`raw` is always stored. The verdict format may change in the next version of the
protocol, and the original text is the only thing the history could then be
rebuilt from.

## Practice and the timebox

Practice runs against a timer set by the topic's `time_box_min`. The countdown
lives in `progress.yaml` as a separate `practice` record rather than in
`attempts[]`: an attempt only appears with an exam verdict, while the timebox
expires earlier — sometimes across several sittings and with no exam at all.

```yaml
practice:
  started_at: "2026-07-28T10:00:00+03:00"   # null — the timer is paused
  spent_sec: 900                            # accumulated, not counting the current run
  expired: true                             # the timebox has already run out
```

The countdown is restored from `started_at`, so moving between screens and
restarting the app do not reset it. Pausing adds the elapsed time to `spent_sec`
and clears `started_at`; a reset erases the record entirely.

Expiry blocks nothing: the timer goes negative and keeps counting, and `expired`
stays recorded even after a pause. It is a measurement, not a prohibition —
practice time says something about the topic and the plan, but not about whether
you may carry on.

## Known limitation

The prompt reduces sycophancy but does not remove it. The only thing that really
holds the grade is executable acceptance of the practice; for theory topics a
`pass` remains the model's judgement. The app does not fix that and does not
pretend to — it merely keeps you from skipping the steps of the protocol.
