import type { ClarificationView, ClarificationsOut } from "../../ipc";
import { toast } from "../../lib/toast";

export async function settle(
  work: () => Promise<ClarificationsOut>,
  changed: (clarifications: ClarificationView[]) => void,
  failed: string,
): Promise<void> {
  try {
    changed((await work()).clarifications);
  } catch {
    toast("error", failed);
  }
}
