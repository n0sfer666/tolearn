const texts = (root) => {
  const found = [];
  const step = (node) => {
    if (node.nodeType === 3) found.push(node);
    else node.childNodes.forEach(step);
  };
  step(root);
  return found;
};

export function selects(window) {
  const stirred = () => window.document.dispatchEvent(new window.Event("selectionchange"));

  const select = (host, id, phrase) => {
    const block = host.querySelector(`#${id}`);
    const node = texts(block).find((text) => text.data.includes(phrase));
    const range = window.document.createRange();
    range.setStart(node, node.data.indexOf(phrase));
    range.setEnd(node, node.data.indexOf(phrase) + phrase.length);
    const selection = window.getSelection();
    selection.removeAllRanges();
    selection.addRange(range);
    stirred();
  };

  const spread = (host, from, head, to, tail) => {
    const start = texts(host.querySelector(`#${from}`)).find((text) => text.data.includes(head));
    const end = texts(host.querySelector(`#${to}`)).find((text) => text.data.includes(tail));
    const range = window.document.createRange();
    range.setStart(start, start.data.indexOf(head));
    range.setEnd(end, end.data.indexOf(tail) + tail.length);
    const selection = window.getSelection();
    selection.removeAllRanges();
    selection.addRange(range);
    stirred();
  };

  const unselect = () => {
    window.getSelection().removeAllRanges();
    stirred();
  };

  const focus = (host, id) => {
    const block = host.querySelector(`#${id}`);
    block.focus();
    window.document.dispatchEvent(new window.Event("focusin"));
  };

  return { select, spread, unselect, focus };
}
