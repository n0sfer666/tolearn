import { Show, createSignal, onMount } from "solid-js";

import Apis from "../components/settings/Apis";
import HarnessFields from "../components/settings/HarnessFields";
import HttpFields from "../components/settings/HttpFields";
import KeyField from "../components/settings/KeyField";
import Kinds from "../components/settings/Kinds";
import Models from "../components/settings/Models";
import type { Locale } from "../i18n";
import type { Dictionary } from "../i18n/ru";
import type {
  AdviceView,
  CheckedView,
  PresetView,
  ProbedView,
  ProviderOut,
  ProviderView,
} from "../ipc";
import { answered, reason, spoken } from "../lib/provider";
import { quiet } from "../lib/ipc";
import { toast } from "../lib/toast";
import type { Transport } from "../lib/ipc";

interface Props {
  text: Dictionary;
  locale: Locale;
  call?: Transport;
}

export default function Provider(props: Props) {
  const call = () => props.call ?? quiet;

  const [draft, setDraft] = createSignal<ProviderView | null>(null);
  const [stored, setStored] = createSignal(false);
  const [key, setKey] = createSignal("");
  const [presets, setPresets] = createSignal<PresetView[]>([]);
  const [checked, setChecked] = createSignal<CheckedView | null>(null);
  const [probed, setProbed] = createSignal<ProbedView | null>(null);
  const [advised, setAdvised] = createSignal<AdviceView[]>([]);
  const [busy, setBusy] = createSignal(false);

  const took = (answer: ProviderOut) => {
    setDraft(answer.provider);
    setStored(answer.has_key);
    setPresets(answer.presets);
    setAdvised(answer.advised);
    setChecked(answer.checked);
    setProbed(answer.probed);
  };

  onMount(() => {
    void (async () => {
      took(
        await call()("provider", {
          save: null,
          key: null,
          forget: false,
          check: false,
          probe: false,
        }),
      );
    })();
  });

  const change = (next: Partial<ProviderView>) => {
    const current = draft();
    if (current === null) return;
    setDraft({ ...current, ...next });
  };

  const send = (check: boolean, probe: boolean, forget: boolean) => {
    const current = draft();
    if (current === null || busy()) return;
    const given = key().trim();
    setBusy(true);
    void (async () => {
      try {
        took(
          await call()("provider", {
            save: current,
            key: given === "" ? null : given,
            forget,
            check,
            probe,
          }),
        );
        setKey("");
        toast("ok", props.text.provider.saved);
      } catch (error) {
        setChecked(null);
        setProbed(null);
        toast("error", reason(error, props.text));
      }
      setBusy(false);
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

          <Kinds
            text={props.text}
            active={current().active}
            onPick={(active) => change({ active })}
          />

          <Show when={current().active === "local"}>
            <Apis
              text={props.text}
              active={current().local.api}
              onPick={(api) => change({ local: spoken(current().local, api) })}
            />
            <HttpFields
              text={props.text}
              value={current().local}
              local
              onChange={(local) => change({ local })}
            />
            <Models
              text={props.text}
              advised={advised()}
              api={current().local.api}
              chosen={current().local.model}
              onPick={(model) => change({ local: { ...current().local, model } })}
            />
          </Show>

          <Show when={current().active === "remote"}>
            <HttpFields
              text={props.text}
              value={current().remote}
              local={false}
              onChange={(remote) => change({ remote })}
            />
            <KeyField
              text={props.text}
              value={key()}
              stored={stored()}
              onChange={setKey}
              onForget={() => send(false, false, true)}
            />
          </Show>

          <Show when={current().active === "harness"}>
            <HarnessFields
              text={props.text}
              value={current().harness}
              presets={presets()}
              onChange={(harness) => change({ harness })}
            />
          </Show>

          <button type="button" data-save disabled={busy()} onClick={() => send(false, false, false)}>
            {props.text.provider.save}
          </button>
          <button type="button" data-check disabled={busy()} onClick={() => send(true, false, false)}>
            {props.text.provider.check}
          </button>
          <button type="button" data-probe disabled={busy()} onClick={() => send(false, true, false)}>
            {props.text.provider.probe}
          </button>

          <Show when={busy()}>
            <p data-working>{props.text.provider.working}</p>
          </Show>

          <Show when={checked()}>
            {(found) => (
              <p data-checked>
                {found().version === null
                  ? `${props.text.provider.checked} ${found().models.join(", ")}`
                  : `${props.text.provider.version} ${found().version}`}
              </p>
            )}
          </Show>

          <Show when={probed()}>
            {(said) => <p data-probed>{answered(said(), props.text)}</p>}
          </Show>
        </article>
      )}
    </Show>
  );
}
