import { For } from "solid-js";

export interface Crumb {
  href: string;
  title: string;
}

interface Props {
  label: string;
  crumbs: Crumb[];
}

export default function Trail(props: Props) {
  const last = () => props.crumbs.length - 1;

  return (
    <nav data-trail aria-label={props.label}>
      <For each={props.crumbs}>
        {(crumb, index) => (
          <a href={crumb.href} data-up={index() === last() ? "" : undefined}>
            {crumb.title}
          </a>
        )}
      </For>
    </nav>
  );
}
