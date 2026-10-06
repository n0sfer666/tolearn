import { Show, createSignal, onMount } from "solid-js";

import Apis from "../components/settings/Apis";
import Drift from "../components/settings/Drift";
import HarnessFields from "../components/settings/HarnessFields";
import HttpFields from "../components/settings/HttpFields";
import Keeping from "../components/settings/Keeping";
import KeyField from "../components/settings/KeyField";
import Kinds from "../components/settings/Kinds";
import Models from "../components/settings/Models";
import { type Locale, hints } from "../i18n";
import type { Dictionary } from "../i18n/ru";
import type {
  AdviceView,
  CheckedView,
  DriftView,
  HarnessView,
  PresetView,
  ProbedView,
  ProviderOut,
  ProviderView,
} from "../ipc";
import { answered, reason, seen, spoken, told } from "../lib/provider";
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
  const advice = () => hints(props.locale);

  const [draft, setDraft] = createSignal<ProviderView | null>(null);
  const [saved, setSaved] = createSignal<ProviderView | null>(null);
  const [stored, setStored] = createSignal(false);
  const [key, setKey] = createSignal("");
  const [presets, setPresets] = createSignal<PresetView[]>([]);
  const [checked, setChecked] = createSignal<CheckedView | null>(null);
  const [probed, setProbed] = createSignal<ProbedView | null>(null);
  const [advised, setAdvised] = createSignal<AdviceView[]>([]);
  const [outdated, setOutdated] = createSignal<DriftView | null>(null);
  const [refusal, setRefusal] = createSignal("");
  const [busy, setBusy] = createSignal(false);

  const took = (answer: ProviderOut) => {
    setDraft(answer.provider);
    setSaved(answer.provider);
    setStored(answer.has_key);
    setPresets(answer.presets);
    setAdvised(answer.advised);
    setChecked(answer.checked);
    setProbed(answer.probed);
    setOutdated(answer.outdated);
  };

  const pristine = () => JSON.stringify(draft()) === JSON.stringify(saved());

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
    setRefusal("");
    setDraft({ ...current, ...next });
  };

  const send = (check: boolean, probe: boolean, forget: boolean) => {
    const current = draft();
    if (current === null || busy()) return;
    const given = key().trim();
    setBusy(true);
    setRefusal("");
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
        setRefusal(told(error));
        toast("error", reason(error, props.text));
      }
      setBusy(false);
    })();
  };

  const resolve = (harness: HarnessView) => {
    change({ harness });
    send(false, false, false);
  };

  const update = () => {
    const provider = draft();
    const found = outdated();
    if (provider === null || found === null) return;
    const known = presets().find((one) => one.id === provider.harness.id);
    if (known === undefined) return;
    resolve({ ...provider.harness, args: [...known.args], dismissed_advice: null });
  };

  const keep = () => {
    const provider = draft();
    const found = outdated();
    if (provider === null || found === null) return;
    resolve({ ...provider.harness, dismissed_advice: found.fingerprint });
  };

  return (
    <Show when={draft()}>
      {(current) => (
        <section data-card="provider">
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
              hints={advice()}
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
              hints={advice()}
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
              hints={advice()}
              value={current().harness}
              presets={presets()}
              onChange={(harness) => change({ harness })}
            />
            <Show when={pristine() && outdated()}>
              {(found) => <Drift text={props.text} found={found()} onUpdate={update} onKeep={keep} />}
            </Show>
          </Show>

          <Keeping text={props.text} on={current().journal} onToggle={(journal) => change({ journal })} />

          <div data-doing>
            <button type="button" data-save disabled={busy()} onClick={() => send(false, false, false)}>
              {props.text.provider.save}
            </button>
            <div class="row">
              <button type="button" data-check disabled={busy()} onClick={() => send(true, false, false)}>
                {props.text.provider.check}
              </button>
              <button type="button" data-probe disabled={busy()} onClick={() => send(false, true, false)}>
                {props.text.provider.probe}
              </button>
            </div>
          </div>

          <Show when={busy()}>
            <p data-working>{props.text.provider.working}</p>
          </Show>

          <Show when={refusal() !== ""}>
            <p data-refusal-title>{props.text.provider.refusal}</p>
            <pre data-refusal>{refusal()}</pre>
          </Show>

          <Show when={checked()}>
            {(found) => <p data-checked>{seen(found(), props.text)}</p>}
          </Show>

          <Show when={probed()}>
            {(said) => <p data-probed>{answered(said(), props.text)}</p>}
          </Show>
        </section>
      )}
    </Show>
  );
}
