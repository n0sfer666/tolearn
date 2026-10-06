import type { Transport } from "./ipc";

export async function acknowledge(call: Transport, since: number): Promise<void> {
  const { outcome } = await call("generation_state", {});
  if (outcome === null || outcome.seen || outcome.work.began < since) return;
  await call("generation_seen", {});
}
