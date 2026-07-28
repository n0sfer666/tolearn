import { Show, createSignal, onMount } from "solid-js";

import type { Locale } from "../i18n";
import type { Dictionary } from "../i18n/ru";
import type { ProviderOut, ProviderView } from "../ipc";
import { transport } from "../lib/ipc";
import type { Transport } from "../lib/ipc";

interface Props {
  text: Dictionary;
  locale: Locale;
  call?: Transport;
}

export default function Provider(props: Props) {
  const call = () => props.call ?? transport;

  const [draft, setDraft] = createSignal<ProviderView | null>(null);
  const [stored, setStored] = createSignal(false);
  const [key, setKey] = createSignal("");
  const [models, setModels] = createSignal<string[] | null>(null);
  const [saved, setSaved] = createSignal(false);
  const [failed, setFailed] = createSignal("");

  const took = (answer: ProviderOut) => {
    setDraft(answer.provider);
    setStored(answer.has_key);
    setModels(answer.checked === null ? null : answer.checked.models);
  };

  onMount(() => {
    void (async () => {
      took(await call()("provider", { save: null, key: null, forget: false, check: false }));
    })();
  });

  const change = (next: Partial<ProviderView>) => {
    const current = draft();
    if (current === null) return;
    setDraft({ ...current, ...next });
    setSaved(false);
  };

  const send = (check: boolean, forget: boolean) => {
    const current = draft();
    if (current === null) return;
    const given = key().trim();
    void (async () => {
      try {
        took(
          await call()("provider", {
            save: current,
            key: given === "" ? null : given,
            forget,
            check,
          }),
        );
        setKey("");
        setFailed("");
        setSaved(true);
      } catch (error) {
        setModels(null);
        setSaved(false);
        setFailed(reason(error, props.text));
      }
    })();
  };

  return (
    <Show when={draft()}>
      {(current) => (
        <article>
          <h2>{props.text.provider.title}</h2>
          <p>{props.text.provider.lead}</p>

          <label>
            <input
              data-enabled
              type="checkbox"
              checked={current().enabled}
              onChange={(event) => change({ enabled: event.currentTarget.checked })}
            />
            {props.text.provider.enabled}
          </label>

          <label>
            {props.text.provider.flavor}
            <select
              data-flavor
              value={current().flavor}
              onChange={(event) => change({ flavor: event.currentTarget.value })}
            >
              <option value="ollama">{props.text.provider.ollama}</option>
              <option value="openai">{props.text.provider.openai}</option>
            </select>
          </label>

          <label>
            {props.text.provider.endpoint}
            <input
              data-endpoint
              type="url"
              value={current().endpoint}
              onInput={(event) => change({ endpoint: event.currentTarget.value })}
            />
          </label>

          <label>
            {props.text.provider.model}
            <input
              data-model
              type="text"
              value={current().model}
              onInput={(event) => change({ model: event.currentTarget.value })}
            />
          </label>

          <label>
            {props.text.provider.key}
            <input
              data-key
              type="password"
              value={key()}
              onInput={(event) => setKey(event.currentTarget.value)}
            />
          </label>
          <p data-stored>
            {stored() ? props.text.provider.keyStored : props.text.provider.keyEmpty}
          </p>
          <Show when={stored()}>
            <button type="button" data-forget onClick={() => send(false, true)}>
              {props.text.provider.forget}
            </button>
          </Show>

          <button type="button" data-save onClick={() => send(false, false)}>
            {props.text.provider.save}
          </button>
          <button type="button" data-check onClick={() => send(true, false)}>
            {props.text.provider.check}
          </button>

          <Show when={models()}>
            {(found) => (
              <p data-checked>
                {props.text.provider.checked} {found().join(", ")}
              </p>
            )}
          </Show>
          <Show when={saved()}>
            <p data-saved>{props.text.provider.saved}</p>
          </Show>
          <Show when={failed() !== ""}>
            <p data-failed>{failed()}</p>
          </Show>
        </article>
      )}
    </Show>
  );
}

function reason(error: unknown, text: Dictionary): string {
  const problems: Record<string, string> = {
    "provider.disabled": text.provider.disabled,
    "provider.no-key": text.provider.noKey,
    "provider.unreachable": text.provider.unreachable,
    "provider.rejected": text.provider.rejected,
    "provider.answered": text.provider.answered,
    "provider.bad-answer": text.provider.badAnswer,
    "provider.vault": text.provider.vault,
  };
  return problems[code(error)] ?? text.provider.failed;
}

function code(error: unknown): string {
  if (error instanceof Object && "code" in error && typeof error.code === "string") {
    return error.code;
  }
  return "";
}
