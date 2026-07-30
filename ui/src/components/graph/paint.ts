import type { Link, Spot } from "./layout";
import type { NodeView } from "../../ipc";

export interface View {
  scale: number;
  x: number;
  y: number;
}

export interface Palette {
  edge: string;
  text: string;
  ring: string;
  paper: string;
  status: Record<string, string>;
}

export interface Scene {
  nodes: readonly NodeView[];
  links: readonly Link[];
  spots: Map<string, Spot>;
  view: View;
  palette: Palette;
  lit: string;
}

const RADIUS = 9;
const LIT = 13;
const LABEL = 13;
const GAP = 6;
const EDGE = 1.4;

export function at(view: View, x: number, y: number): Spot {
  return { x: (x - view.x) / view.scale, y: (y - view.y) / view.scale };
}

export function near(
  nodes: readonly NodeView[],
  spots: Map<string, Spot>,
  scale: number,
  spot: Spot,
): string {
  const reach = (LIT + GAP) / scale;
  const found = nodes.find((node) => {
    const place = spots.get(node.id);
    return (
      place !== undefined &&
      Math.hypot(place.x - spot.x, place.y - spot.y) <= reach
    );
  });
  return found?.id ?? "";
}

export function paint(
  context: CanvasRenderingContext2D,
  scene: Scene,
  size: Spot,
): void {
  const { view, palette } = scene;
  context.setTransform(1, 0, 0, 1, 0, 0);
  context.fillStyle = palette.paper;
  context.fillRect(0, 0, size.x, size.y);
  context.setTransform(view.scale, 0, 0, view.scale, view.x, view.y);
  wires(context, scene);
  for (const node of scene.nodes) {
    dot(context, scene, node);
  }
}

function wires(context: CanvasRenderingContext2D, scene: Scene): void {
  context.strokeStyle = scene.palette.edge;
  context.lineWidth = EDGE / scene.view.scale;
  context.beginPath();
  for (const link of scene.links) {
    const from = scene.spots.get(link.from);
    const to = scene.spots.get(link.to);
    if (from === undefined || to === undefined) continue;
    context.moveTo(from.x, from.y);
    context.lineTo(to.x, to.y);
  }
  context.stroke();
}

function dot(
  context: CanvasRenderingContext2D,
  scene: Scene,
  node: NodeView,
): void {
  const spot = scene.spots.get(node.id);
  if (spot === undefined) return;
  const size = node.id === scene.lit ? LIT : RADIUS;
  context.beginPath();
  context.arc(spot.x, spot.y, size, 0, Math.PI * 2);
  context.fillStyle = scene.palette.status[node.status] ?? scene.palette.edge;
  context.fill();
  context.lineWidth = 1 / scene.view.scale;
  context.strokeStyle = scene.palette.ring;
  context.stroke();
  name(context, scene, node.title, spot, size);
}

function name(
  context: CanvasRenderingContext2D,
  scene: Scene,
  title: string,
  spot: Spot,
  size: number,
): void {
  context.fillStyle = scene.palette.text;
  context.font = `${LABEL}px system-ui, sans-serif`;
  context.textAlign = "center";
  context.textBaseline = "top";
  context.fillText(title, spot.x, spot.y + size + GAP);
}
