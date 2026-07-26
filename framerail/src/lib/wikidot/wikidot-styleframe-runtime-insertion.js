import { STYLEFRAME_PRELOADED } from "./wikidot-styleframe-contract.js"

export const STYLEFRAME_INSERTION_RUNTIME_SOURCE = `  const markOwned = (element, id) => {
    element.dataset.wikidotStyleFrame = marker;
    element.dataset.wikidotStyleOwner = owner;
    element.dataset.wikidotStylePriority = priority;
    element.dataset.wikidotStyleId = id;
  };
  const insertMarked = (element, id) => {
    markOwned(element, id);
    const laterStyle = markedStyleNodes().find((node) => stylePriority(node) > priorityValue);
    if (laterStyle) {
      head.insertBefore(element, laterStyle);
    } else {
      head.appendChild(element);
    }
  };
  let insertedStyle = false;
  themes.forEach((href, index) => {
    const preloadedLink = Array.from(
      head.querySelectorAll('link[data-wikidot-style-preloaded]')
    ).find((candidate) =>
      candidate.href === new URL(href, targetDocument.baseURI).href &&
      candidate.dataset.wikidotStylePriority === priority
    );
    const link = preloadedLink || targetDocument.createElement("link");
    if (preloadedLink) {
      delete link.dataset.${STYLEFRAME_PRELOADED};
      markOwned(link, \`theme-\${index}\`);
      return;
    }
    link.rel = "stylesheet";
    link.href = href;
    insertMarked(link, \`theme-\${index}\`);
    insertedStyle = true;
  });
  if (css.trim().length > 0) {
    const preloadedStyle = Array.from(
      head.querySelectorAll('style[data-wikidot-style-preloaded]')
    ).find((candidate) =>
      candidate.textContent === css &&
      candidate.dataset.wikidotStylePriority === priority
    );
    const style = preloadedStyle || targetDocument.createElement("style");
    if (preloadedStyle) {
      delete style.dataset.${STYLEFRAME_PRELOADED};
      markOwned(style, "inline-css");
    } else {
      style.textContent = css;
      insertMarked(style, "inline-css");
      insertedStyle = true;
    }
  }
  if (insertedStyle) scheduleStyleFrameOrderRestore();`
