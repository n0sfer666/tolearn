import { copy as toClipboard } from "../../lib/clipboard";
import { toast } from "../../lib/toast";

export interface Words {
  copied: string;
  manual: string;
  label: string;
}

export type Copier = (text: string) => Promise<void>;

interface Props {
  text: string;
  words: Words;
  copy?: Copier;
}

export default function Snip(props: Props) {
  const take = () => {
    void (async () => {
      try {
        await (props.copy ?? toClipboard)(props.text);
        toast("ok", props.words.copied);
      } catch {
        toast("warn", props.words.manual);
      }
    })();
  };

  return (
    <button
      type="button"
      data-snip
      aria-label={`${props.words.label}: ${props.text}`}
      title={props.words.label}
      onClick={take}
    >
      <code>{props.text}</code>
    </button>
  );
}
