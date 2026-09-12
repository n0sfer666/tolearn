# Exam protocol

[Русский](../ru/protocol.md) · **English**

The app's contract. This document describes what `tolearn` does with an exam
verdict.

This document describes the v1 exam. The app does not take exams right now: the
exam comes back on top of the v2 format ([format.md](../format.md)), and this
contract will then be rewritten for stages instead of topics. What the
**examining model** does is up to its prompt. What the **app** does is up to
this document — otherwise the app's behaviour would depend on the contents of an
imported program.

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

## The `practice` record

Before v2, practice ran against a timer set by the topic's `time_box_min`, and
the countdown was kept in `progress.yaml` as a separate `practice` record
(`started_at`, `spent_sec`, `expired`). The timer is gone: the app neither
writes nor reads this record, and one already written is carried over word for
word on save, like any unfamiliar field. `time_box_min` stays as a guide on the
practice screen.

## Known limitation

The prompt reduces sycophancy but does not remove it. The only thing that really
holds the grade is executable acceptance of the practice; for theory topics a
`pass` remains the model's judgement. The app does not fix that and does not
pretend to — it merely keeps you from skipping the steps of the protocol.
