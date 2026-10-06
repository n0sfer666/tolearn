import { grab } from "./grab";

export function regain(when: () => boolean): (element: HTMLElement) => void {
  return (element) => {
    if (when()) grab(element);
  };
}
