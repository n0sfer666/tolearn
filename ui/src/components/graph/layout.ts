import type { NodeView } from "../../ipc";

export interface Spot {
  x: number;
  y: number;
}

export interface Link {
  from: string;
  to: string;
}

const TICKS = 300;
const SPREAD = 150;
const NUDGE = 20;
const SPACE = 110;
const HEAT = 200;
const HOME = 4;
const NEAR = 0.01;
const TURN = 2.399_963;

export function links(nodes: readonly NodeView[]): Link[] {
  const here = new Set(nodes.map((node) => node.id));
  return nodes.flatMap((node) =>
    node.depends_on
      .filter((from) => here.has(from))
      .map((from) => ({ from, to: node.id })),
  );
}

export function laid(nodes: readonly NodeView[]): Map<string, Spot> {
  const spots = seeded(nodes);
  const drawn = links(nodes);
  for (let tick = 0; tick < TICKS; tick += 1) {
    relax(nodes, drawn, spots, HEAT * (1 - tick / TICKS));
  }
  return spots;
}

function seeded(nodes: readonly NodeView[]): Map<string, Spot> {
  const middle = (Math.max(...nodes.map((node) => node.layer), 0) + 1) / 2;
  return new Map(
    nodes.map((node, index) => [
      node.id,
      {
        x: (node.layer + 1 - middle) * SPREAD + Math.cos(index * TURN) * NUDGE,
        y: Math.sin(index * TURN) * SPREAD * Math.sqrt(index + 1) * 0.6,
      },
    ]),
  );
}

function relax(
  nodes: readonly NodeView[],
  drawn: readonly Link[],
  spots: Map<string, Spot>,
  heat: number,
): void {
  const shifts = new Map(nodes.map((node) => [node.id, { x: 0, y: 0 }]));
  push(nodes, spots, shifts);
  pull(drawn, spots, shifts);
  for (const node of nodes) {
    const spot = spots.get(node.id);
    const shift = shifts.get(node.id);
    if (spot === undefined || shift === undefined) continue;
    step(spot, shift, heat);
  }
}

function step(spot: Spot, shift: Spot, heat: number): void {
  const dx = shift.x - spot.x * HOME;
  const dy = shift.y - spot.y * HOME;
  const far = Math.max(Math.hypot(dx, dy), NEAR);
  const take = Math.min(far, heat);
  spot.x += (dx / far) * take;
  spot.y += (dy / far) * take;
}

function push(
  nodes: readonly NodeView[],
  spots: Map<string, Spot>,
  shifts: Map<string, Spot>,
): void {
  for (let one = 0; one < nodes.length; one += 1) {
    for (let two = one + 1; two < nodes.length; two += 1) {
      apart(nodes[one].id, nodes[two].id, spots, shifts);
    }
  }
}

function apart(
  one: string,
  two: string,
  spots: Map<string, Spot>,
  shifts: Map<string, Spot>,
): void {
  const here = spots.get(one);
  const there = spots.get(two);
  const mine = shifts.get(one);
  const yours = shifts.get(two);
  if (
    here === undefined ||
    there === undefined ||
    mine === undefined ||
    yours === undefined
  )
    return;
  const dx = here.x - there.x;
  const dy = here.y - there.y;
  const far = Math.max(Math.hypot(dx, dy), NEAR);
  const force = (SPACE * SPACE) / far;
  mine.x += (dx / far) * force;
  mine.y += (dy / far) * force;
  yours.x -= (dx / far) * force;
  yours.y -= (dy / far) * force;
}

function pull(
  drawn: readonly Link[],
  spots: Map<string, Spot>,
  shifts: Map<string, Spot>,
): void {
  for (const link of drawn) {
    const here = spots.get(link.from);
    const there = spots.get(link.to);
    const mine = shifts.get(link.from);
    const yours = shifts.get(link.to);
    if (
      here === undefined ||
      there === undefined ||
      mine === undefined ||
      yours === undefined
    ) {
      continue;
    }
    const dx = there.x - here.x;
    const dy = there.y - here.y;
    const far = Math.max(Math.hypot(dx, dy), NEAR);
    const force = (far * far) / SPACE;
    mine.x += (dx / far) * force;
    mine.y += (dy / far) * force;
    yours.x -= (dx / far) * force;
    yours.y -= (dy / far) * force;
  }
}
