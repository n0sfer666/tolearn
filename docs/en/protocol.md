# Exam protocol

[Русский](../ru/protocol.md) · **English**

The app's contract. This document describes what `tolearn` does with a stage
exam verdict ([ADR-026](../adr/026-learning-cycle.md)). What the **examining
model** does is up to its prompt. What the **app** does is up to this
document — otherwise the app's behaviour would depend on the contents of a
program.

The verdict arrives the same way in a written exam, when the answers go to the
model in one request ([ADR-017](../adr/017-written-exam-only.md)), and in
copypaste, when the same prompt is sent to someone else's chat and the answer
is pasted back. Reading the verdict calls no model and touches no network.

## The written exam

Under each question on the stage screen there is an answer field. The draft is
written to the stage's state after a pause in typing, on leaving the field and
on closing the window, so it survives a restart; an empty field removes the
draft.

"Submit" gathers the answers together with the stage's reference `answer`s,
the program's level and language into one prompt; a question with no answer
goes as "no answer".

1. All fields empty — refused with "answer at least one question", no model is
   called.
2. A remote provider — an HTTP service or a CLI harness — needs the network,
   and the app checks it before the call. A local model needs no check.
3. One model call, and its answer is read as the verdict below. If the verdict
   cannot be read, the model gets its answer back with the reason once and
   repairs it. If that fails too — refused with the reason, and nothing is
   written to the state.
4. Every call, the repair included, is written to the request log as the
   `exam` step.

An accepted verdict becomes a `written` attempt with the model's name, and each
question shows its result and what was missed.

The exam can also be taken through any other chat, with no provider and no
network. "Copy the prompt" puts the same prompt the model would get, with the
person's answers, on the clipboard. The chat's whole reply goes into "Paste the
reply" and passes the same parsing, without the repair: a refusal shows its
reason from the "When a verdict is rejected" table and the state stays as it
was. An accepted verdict becomes a `copypaste` attempt with no model, and
nothing goes to the request log.

## The verdict

The app takes the **last** top-level JSON object from the answer text — inside
a ` ```json ` fence or without one. The whole answer is pasted: demanding that
a person dig the block out by hand guarantees mistakes. Objects nested inside
another object do not count as a verdict of their own.

```json
{
  "stage": "voices",
  "per_question": [
    {"id": "q1", "result": "ok"},
    {"id": "q2", "result": "partial",
     "missed": ["why a 25 % and a 75 % duty cycle sound the same"]},
    {"id": "q3", "result": "miss", "missed": ["the whole answer"]}
  ]
}
```

- `stage` — the `id` of the stage being examined;
- `per_question` — exactly one row for each question of the stage, in any
  order;
- `result` — `ok`, `partial` or `miss`, in lower case;
- `missed` — what the answer missed, as lines of text. Required and non-empty
  for a question not passed (`partial`, `miss`), optional for a passed one.

The app neither reads nor stores any other field.

## When a verdict is rejected

| Cause | What the person sees |
|---|---|
| the text holds no JSON object | "the text holds no JSON block with a verdict" |
| `stage` of another stage | "the verdict of stage `…` was pasted into the wrong stage" |
| no `stage` or `per_question`, a field of the wrong type | "the verdict could not be read" and the reason |
| `result` other than `ok`, `partial` or `miss` | "the verdict could not be read" and the reason |
| a question not passed has no `missed` or an empty one | "the verdict could not be read" and the reason |
| the questions do not match the stage's questions | which are missing, which are stray, which are graded twice |

A rejected verdict is not written to the state. There is no partial parsing: a
verdict that does not cover the whole stage cannot be told apart from
eyeballing a grade.

## Verdict → status

An accepted verdict becomes an attempt on the stage.

| Verdict | Status | Also |
|---|---|---|
| `ok` on every question | "passed", exam taken | `passed` — the attempt's date, by `exam` |
| any `partial` or `miss` | "started" | the questions show what was missed |

- A failed attempt does not take "passed" away: a late failure is a reason to
  repeat, not a loss of what was passed.
- An exam passed after a skip changes the mark to `exam` and sets the
  attempt's date.
- An exam passed again does not change the date of the first pass.
- An attempt on a stage not yet opened opens it with the attempt's date.

A question not passed is one whose `result` is not `ok` in the **last**
attempt of its stage. These questions are shown on the stage screen along with
`missed` and go into the following prompts as ADR-026 describes under "What
the exam passes on".

## Skipping

A stage can be marked passed without an exam: a person may have known the
material already or put the check off. A skip is written as `passed.by: skip`,
it has no attempt, and the summary counts such stages separately — "without an
exam" — otherwise the "passed" number stops meaning anything.

"Skip the check" at the end of a stage records the skip with the day it was
pressed, opens the stage if it was not open yet, and leads to the fork. A stage
already passed is left alone: a passed exam stays passed and its date does not
change. The exam can still be taken after a skip, by the rules above. A skip
adds nothing to the fork and next-stage prompts: it has no questions not
passed.

## The shape of an attempt

An attempt lives in `stages.<key>.attempts[]` of `state.yaml`
([format.md](../format.md)). The verdict is stored verbatim, next to what the
app knows and the model does not:

```yaml
attempts:
  - "on": 2026-09-14
    by: written          # written | copypaste
    model: claude-opus-5 # written only
    per_question:
      - id: q1
        result: ok
      - id: q2
        result: partial
        missed:
          - why a 25 % and a 75 % duty cycle sound the same
```

- `on` — the attempt's date;
- `by` — how: a written exam in the app, or copypaste;
- `model` — the model that took the exam; with copypaste the app does not know
  it, and the field is absent;
- `per_question` — the verdict's rows in the order of the answer, `missed`
  word for word.

There is no `raw` field: the state schema is closed, and `per_question` is the
whole of the verdict the app reads. Attempts are only appended; earlier ones
are never rewritten or removed.

## Known limitation

The prompt reduces sycophancy but does not remove it. The only thing that
really holds the grade is executable acceptance of the practice, and the person
ticks that off ([ADR-006](../adr/006-human-verdict.md)). For a stage's
questions an `ok` remains the model's judgement. The app does not fix that and
does not pretend to — it merely keeps you from skipping the steps of the
protocol.
