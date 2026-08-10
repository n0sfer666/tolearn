import type { Dictionary } from "../../i18n/ru";
import type { OfflineCostOut } from "../../ipc";
import { megabytes } from "../../lib/size";

interface Props {
  text: Dictionary;
  cost: OfflineCostOut;
  onGo: () => void;
  onBack: () => void;
}

export default function Price(props: Props) {
  return (
    <div data-unload-price>
      <p data-unload-tight>{props.text.offline.tight}</p>
      <ul>
        <li data-price-pieces>
          {props.text.offline.pieces}: {props.cost.materials}
        </li>
        <li data-price-kept>
          {props.text.offline.kept}: {props.cost.held}
        </li>
        <li data-price-used>
          {props.text.offline.used}: {megabytes(props.cost.used)} / {megabytes(props.cost.budget)}{" "}
          {props.text.offline.bytes}
        </li>
        <li data-price-need>
          {props.text.offline.need}: {megabytes(props.cost.need)} {props.text.offline.bytes}
        </li>
        <li data-price-spare>
          {props.text.offline.spare}: {megabytes(props.cost.spare)} {props.text.offline.bytes}
        </li>
      </ul>
      <button type="button" data-unload-go onClick={props.onGo}>
        {props.text.offline.go}
      </button>
      <button type="button" data-unload-back onClick={props.onBack}>
        {props.text.offline.back}
      </button>
    </div>
  );
}
