import { createMemo, createSignal, onCleanup, onMount } from "solid-js";

import { laid, links } from "./layout";
import type { Spot } from "./layout";
import { at, near, paint } from "./paint";
import type { Palette, View } from "./paint";
import { palette } from "./palette";
import type { NodeView } from "../../ipc";

interface Props {
  nodes: NodeView[];
  label: string;
  onOpen: (id: string) => void;
}

const PAD = 60;
const CLOSEST = 0.3;
const FARTHEST = 3;
const WHEEL = 0.0015;
const SLIP = 4;

export default function Canvas(props: Props) {
  let board: HTMLCanvasElement | undefined;
  const spots = createMemo(() => laid(props.nodes));
  const wires = createMemo(() => links(props.nodes));
  const [view, setView] = createSignal<View>({ scale: 1, x: 0, y: 0 });
  const [lit, setLit] = createSignal("");

  let held = "";
  let panning = false;
  let slipped = 0;
  let inked: Palette | null = null;
  let themed = "";

  const inks = (): Palette => {
    const now = board?.ownerDocument.documentElement.dataset.theme ?? "";
    if (inked === null || now !== themed) {
      inked = palette(board?.parentElement ?? board ?? document.body);
      themed = now;
    }
    return inked;
  };

  const draw = () => {
    const canvas = board;
    const context = canvas?.getContext("2d");
    if (canvas === undefined || !context) return;
    const ratio = canvas.ownerDocument.defaultView?.devicePixelRatio ?? 1;
    const size = { x: canvas.clientWidth, y: canvas.clientHeight };
    canvas.width = Math.round(size.x * ratio);
    canvas.height = Math.round(size.y * ratio);
    const seen = view();
    paint(
      context,
      {
        nodes: props.nodes,
        links: wires(),
        spots: spots(),
        view: {
          scale: seen.scale * ratio,
          x: seen.x * ratio,
          y: seen.y * ratio,
        },
        palette: inks(),
        lit: lit(),
        ratio,
      },
      { x: canvas.width, y: canvas.height },
    );
  };

  const fit = () => {
    const canvas = board;
    if (canvas === undefined || props.nodes.length === 0) return;
    const places = [...spots().values()];
    const left = Math.min(...places.map((spot) => spot.x));
    const right = Math.max(...places.map((spot) => spot.x));
    const top = Math.min(...places.map((spot) => spot.y));
    const bottom = Math.max(...places.map((spot) => spot.y));
    const wide = Math.max(right - left, 1) + PAD * 2;
    const tall = Math.max(bottom - top, 1) + PAD * 2;
    const scale = Math.min(
      canvas.clientWidth / wide,
      canvas.clientHeight / tall,
      FARTHEST,
    );
    setView({
      scale,
      x: canvas.clientWidth / 2 - ((left + right) / 2) * scale,
      y: canvas.clientHeight / 2 - ((top + bottom) / 2) * scale,
    });
  };

  const where = (event: PointerEvent | WheelEvent): Spot => {
    const box = board?.getBoundingClientRect();
    return {
      x: event.clientX - (box?.left ?? 0),
      y: event.clientY - (box?.top ?? 0),
    };
  };

  const hit = (spot: Spot) =>
    near(props.nodes, spots(), view().scale, at(view(), spot.x, spot.y));

  const took = (event: PointerEvent) => {
    const spot = where(event);
    held = hit(spot);
    panning = held === "";
    slipped = 0;
    board?.setPointerCapture(event.pointerId);
  };

  const moved = (event: PointerEvent) => {
    const spot = where(event);
    if (held === "" && !panning) {
      setLit(hit(spot));
      draw();
      return;
    }
    slipped += Math.abs(event.movementX) + Math.abs(event.movementY);
    if (panning) shift(event.movementX, event.movementY);
    if (held !== "") drag(event.movementX, event.movementY);
    draw();
  };

  const dropped = (event: PointerEvent) => {
    board?.releasePointerCapture(event.pointerId);
    if (held !== "" && slipped < SLIP) props.onOpen(held);
    held = "";
    panning = false;
  };

  const shift = (dx: number, dy: number) => {
    const seen = view();
    setView({ scale: seen.scale, x: seen.x + dx, y: seen.y + dy });
  };

  const drag = (dx: number, dy: number) => {
    const spot = spots().get(held);
    if (spot === undefined) return;
    spot.x += dx / view().scale;
    spot.y += dy / view().scale;
  };

  const zoom = (event: WheelEvent) => {
    event.preventDefault();
    const seen = view();
    const spot = where(event);
    const step = Math.exp(-event.deltaY * WHEEL);
    const scale = Math.min(Math.max(seen.scale * step, CLOSEST), FARTHEST);
    const ratio = scale / seen.scale;
    setView({
      scale,
      x: spot.x - (spot.x - seen.x) * ratio,
      y: spot.y - (spot.y - seen.y) * ratio,
    });
    draw();
  };

  onMount(() => {
    fit();
    draw();
    const window = board?.ownerDocument.defaultView;
    const again = () => {
      fit();
      draw();
    };
    window?.addEventListener("resize", again);
    onCleanup(() => window?.removeEventListener("resize", again));
  });

  return (
    <canvas
      data-map
      role="img"
      aria-label={props.label}
      ref={board}
      onPointerDown={took}
      onPointerMove={moved}
      onPointerUp={dropped}
      onPointerLeave={() => setLit("")}
      onWheel={zoom}
    />
  );
}
