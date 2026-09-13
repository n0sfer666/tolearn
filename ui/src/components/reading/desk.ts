import type { Dictionary } from "../../i18n/ru";
import type { CheckClaimOut } from "../../ipc";
import type { Transport } from "../../lib/ipc";
import { toast } from "../../lib/toast";
import { told } from "../../lib/told";

export interface Spot {
  program: string;
  node: string;
  stage: string;
}

export interface Desk {
  ticks: () => string[];
  workdir: () => string | null;
  tick: (claim: string, on: boolean) => Promise<boolean>;
  choose: () => Promise<void>;
  check: (claim: string) => Promise<CheckClaimOut>;
  reason: (failure: unknown) => string;
}

export interface Station {
  call: Transport;
  pick: () => Promise<string | null>;
  at: () => Spot;
  text: Dictionary;
  ticks: () => string[];
  workdir: () => string | null;
  ticked: (ticks: string[]) => void;
  chosen: (workdir: string) => void;
}

export function desk(station: Station): Desk {
  const reason = (failure: unknown) =>
    told(
      failure,
      new Map([
        ["workdir.absent", station.text.stage.gone],
        ["workdir.inside", station.text.stage.inside],
        ["workdir.unset", station.text.stage.unset],
      ]),
    );
  return {
    ticks: station.ticks,
    workdir: station.workdir,
    reason,
    tick: async (claim, on) => {
      try {
        const out = await station.call("tick", { ...station.at(), claim, on });
        station.ticked(out.ticks);
        return true;
      } catch (failure) {
        toast("error", reason(failure));
        return false;
      }
    },
    choose: async () => {
      try {
        const path = await station.pick();
        if (path === null) return;
        const out = await station.call("workdir", { program: station.at().program, path });
        station.chosen(out.workdir);
      } catch (failure) {
        toast("error", reason(failure));
      }
    },
    check: (claim) => station.call("check_claim", { ...station.at(), claim }),
  };
}
