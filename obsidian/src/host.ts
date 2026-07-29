import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";

import { programs } from "./registry.ts";
import type { World } from "./view.ts";

const IDENTITY = "identity.age";
const REGISTRY = "registry.yaml";
const NOTES = "notes";

export function world(data: string, today: string): World {
  return {
    sealed: existsSync(join(data, NOTES, IDENTITY)),
    programs: programs(text(join(data, REGISTRY)) ?? ""),
    progress: (path: string) =>
      text(join(path, "progress.yaml")) ?? text(join(path, "progress.json")),
    today,
  };
}

export function text(path: string): string | null {
  try {
    return readFileSync(path, "utf8");
  } catch {
    return null;
  }
}
