import { Show, createSignal, onMount } from "solid-js";

import type { Dictionary } from "../i18n/ru";
import type { EncryptionOut } from "../ipc";
import { transport } from "../lib/ipc";
import type { Transport } from "../lib/ipc";

interface Props {
  text: Dictionary;
  call?: Transport;
}

export default function Encryption(props: Props) {
  const call = () => props.call ?? transport;

  const [view, setView] = createSignal<EncryptionOut | null>(null);
  const [phrase, setPhrase] = createSignal("");
  const [warned, setWarned] = createSignal(false);
  const [busy, setBusy] = createSignal(false);
  const [failed, setFailed] = createSignal("");

  onMount(() => {
    void (async () => {
      setView(await call()("encryption", { enable: null, phrase: null }));
    })();
  });

  const switched = (enable: boolean) => {
    setBusy(true);
    setFailed("");
    void (async () => {
      try {
        setView(await call()("encryption", { enable, phrase: phrase() }));
        setPhrase("");
        setWarned(false);
      } catch (error) {
        setFailed(error instanceof Error ? error.message : props.text.encryption.failed);
      } finally {
        setBusy(false);
      }
    })();
  };

  return (
    <Show when={view()}>
      {(current) => (
        <section>
          <h2>{props.text.encryption.title}</h2>
          <p>{props.text.encryption.lead}</p>

          <Show when={current().external}>
            <p data-external>{props.text.encryption.external}</p>
          </Show>

          <Show
            when={current().enabled}
            fallback={
              <Show when={!current().external}>
                <p data-loss>{props.text.encryption.loss}</p>
                <label>
                  {props.text.encryption.phrase}
                  <input
                    data-phrase
                    type="password"
                    value={phrase()}
                    onInput={(event) => setPhrase(event.currentTarget.value)}
                  />
                </label>
                <label>
                  <input
                    data-warned
                    type="checkbox"
                    checked={warned()}
                    onChange={(event) => setWarned(event.currentTarget.checked)}
                  />
                  {props.text.encryption.understood}
                </label>
                <button
                  type="button"
                  data-enable
                  disabled={busy() || !warned() || phrase().length === 0}
                  onClick={() => switched(true)}
                >
                  {busy() ? props.text.encryption.working : props.text.encryption.enable}
                </button>
              </Show>
            }
          >
            <p data-enabled>{props.text.encryption.enabled}</p>
            <button type="button" data-disable disabled={busy()} onClick={() => switched(false)}>
              {busy() ? props.text.encryption.working : props.text.encryption.disable}
            </button>
          </Show>

          <Show when={failed().length > 0}>
            <p data-failed role="status">
              {failed()}
            </p>
          </Show>
        </section>
      )}
    </Show>
  );
}
