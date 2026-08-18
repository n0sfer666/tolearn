import { bound } from "./bound.ts";
import { deepLink } from "./link.ts";
import { state } from "./progress.ts";
import type { Program } from "./registry.ts";

export type Shown =
  | { kind: "foreign" }
  | { kind: "sealed" }
  | { kind: "no-program"; roadmap: string }
  | { kind: "no-topic"; roadmap: string; topic: string }
  | {
      kind: "topic";
      roadmap: string;
      topic: string;
      title: string;
      status: string;
      overdue: boolean;
      link: string;
    };

export interface World {
  sealed: boolean;
  programs: Program[];
  progress: (path: string) => string | null;
  today: string;
}

export function shown(note: string, world: World): Shown {
  const note_bound = bound(note);
  if (note_bound === null) {
    return { kind: "foreign" };
  }
  const { roadmap, topic } = note_bound;
  if (world.sealed) {
    return { kind: "sealed" };
  }
  const program = world.programs.find((listed) => listed.id === roadmap);
  if (program === undefined) {
    return { kind: "no-program", roadmap };
  }
  const source = world.progress(program.path);
  const found = source === null ? null : state(source, topic);
  if (found === null) {
    return { kind: "no-topic", roadmap, topic };
  }
  return {
    kind: "topic",
    roadmap,
    topic,
    title: program.title,
    status: settled(found.status),
    overdue: found.due !== null && found.due <= world.today,
    link: deepLink(roadmap, topic),
  };
}

function settled(status: string): string {
  return status === "blocked" ? "todo" : status;
}
