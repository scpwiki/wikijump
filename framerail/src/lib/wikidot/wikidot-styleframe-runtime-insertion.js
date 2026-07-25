import { STYLEFRAME_PRELOADED } from "./wikidot-styleframe-contract.js"

export const STYLEFRAME_INSERTION_RUNTIME_SOURCE = `  const appendMarked = (element, id) => {
    element.dataset.wikidotStyleFrame = marker;
    element.dataset.wikidotStyleOwner = owner;
    element.dataset.wikidotStylePriority = priority;
    element.dataset.wikidotStyleId = id;
    const laterStyle = markedStyleNodes().find((node) => stylePriority(node) > priorityValue);
    if (laterStyle) {
      head.insertBefore(element, laterStyle);
    } else {
      head.appendChild(element);
    }
  };
  themes.forEach((href, index) => {
    const link = Array.from(
      head.querySelectorAll('link[data-wikidot-style-preloaded]')
    ).find((candidate) =>
      candidate.href === new URL(href, targetDocument.baseURI).href &&
      candidate.dataset.wikidotStylePriority === priority
    ) || targetDocument.createElement("link");
    delete link.dataset.${STYLEFRAME_PRELOADED};
    link.rel = "stylesheet";
    link.href = href;
    appendMarked(link, \`theme-\${index}\`);
  });
  if (css.trim().length > 0) {
    const style = targetDocument.createElement("style");
    style.textContent = css;
    appendMarked(style, "inline-css");
  }
  scheduleStyleFrameOrderRestore();`
