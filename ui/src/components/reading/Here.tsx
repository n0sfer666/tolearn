import type { JSX } from "solid-js";

interface Props {
  here: string;
  resume: string;
  href: string;
}

export default function Here(props: Props): JSX.Element {
  return (
    <>
      <span data-here>{props.here}</span>
      <a href={props.href} data-resume data-action>
        {props.resume}
      </a>
    </>
  );
}
