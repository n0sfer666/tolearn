import type { Dictionary } from "../../i18n/ru";
import type { StageIn } from "../../ipc";
import { nextHref } from "../../lib/links";

interface Props {
  text: Dictionary;
  locale: string;
  at: StageIn;
  go?: (href: string) => void;
}

export default function Fork(props: Props) {
  const href = () => nextHref(props.locale, props.at);

  const leave = (event: MouseEvent) => {
    const go = props.go;
    if (go === undefined) return;
    event.preventDefault();
    go(href());
  };

  return (
    <a href={href()} data-fork data-action onClick={leave}>
      {props.text.generate.fork}
    </a>
  );
}
