import {
  safeStyleFrameScriptJson,
  STYLEFRAME_MARKER,
  STYLEFRAME_PRELOADED,
  STYLEFRAME_REGISTRY
} from "./wikidot-styleframe-contract.js"

/**
 * @param {{ priority: string; themes: string[]; css: string }} input
 * @returns {string}
 */
export const buildWikidotStyleFrameRuntime = ({ priority, themes, css }) => {
  return `(() => {
  const marker = ${safeStyleFrameScriptJson(STYLEFRAME_MARKER)};
  const priority = ${safeStyleFrameScriptJson(priority)};
  const priorityNumber = Number.parseFloat(priority);
  const priorityValue = Number.isFinite(priorityNumber) ? priorityNumber : 0;
  const themes = ${safeStyleFrameScriptJson(themes)};
  const css = ${safeStyleFrameScriptJson(css)};
  const targetWindow = window.parent && window.parent !== window
    ? window.parent
    : window;
  const targetDocument = targetWindow.document;
  const frameElement = window.frameElement || null;
  const registryKey = ${safeStyleFrameScriptJson(STYLEFRAME_REGISTRY)};
  const registry = targetWindow[registryKey] || (targetWindow[registryKey] = {
    nextOwnerId: 1,
    owners: new Map()
  });
  const removeOwner = (owner) => {
    const registration = registry.owners.get(owner);
    targetDocument.querySelectorAll(
      '[data-wikidot-style-owner="' + owner + '"]'
    ).forEach((node) => {
      if (node !== registration?.frame) node.remove();
    });
    if (registration?.frame?.dataset.wikidotStyleOwner === owner) {
      delete registration.frame.dataset.wikidotStyleOwner;
    }
    registration?.observer?.disconnect();
    registry.owners.delete(owner);
  };
  registry.owners.forEach((registration, owner) => {
    if (registration.frame && !registration.frame.isConnected) removeOwner(owner);
  });
  const previousOwner = frameElement?.dataset.wikidotStyleOwner;
  if (previousOwner) removeOwner(previousOwner);
  const owner = marker + "-" + registry.nextOwnerId++;
  if (frameElement) frameElement.dataset.wikidotStyleOwner = owner;
  registry.owners.set(owner, { frame: frameElement, observer: null });
  const cleanup = () => removeOwner(owner);
  window.addEventListener?.("pagehide", cleanup, { once: true });
  window.addEventListener?.("unload", cleanup, { once: true });
  if (frameElement && typeof targetWindow.MutationObserver === "function") {
    const observer = new targetWindow.MutationObserver(() => {
      if (!frameElement.isConnected) cleanup();
    });
    observer.observe(targetDocument.documentElement, { childList: true, subtree: true });
    registry.owners.get(owner).observer = observer;
  }
  const head = targetDocument.head || targetDocument.documentElement;
  const markedStyleNodes = () => Array.from(
    head.querySelectorAll('[data-wikidot-style-frame="' + marker + '"]')
  );
  const generatedCssNodes = () => Array.from(
    head.querySelectorAll("[data-wikijump-generated-css]")
  );
  const stylePriority = (node) => {
    const value = Number.parseFloat(node.dataset.wikidotStylePriority || "");
    return Number.isFinite(value) ? value : 0;
  };
  const restoreStyleFrameOrder = () => {
    const desiredTail = [
      ...markedStyleNodes().sort(
        (left, right) => stylePriority(left) - stylePriority(right)
      ),
      ...generatedCssNodes()
    ];
    const currentNodes = Array.from(head.children);
    const tailOffset = currentNodes.length - desiredTail.length;
    const alreadyOrdered =
      tailOffset >= 0 &&
      desiredTail.every((node, index) => currentNodes[tailOffset + index] === node);
    if (alreadyOrdered) return;
    desiredTail.forEach((node) => head.appendChild(node));
  };
  const scheduleStyleFrameOrderRestore = () => {
    restoreStyleFrameOrder();
    setTimeout(restoreStyleFrameOrder, 0);
    setTimeout(restoreStyleFrameOrder, 250);
    if (typeof targetDocument.defaultView?.requestAnimationFrame === "function") {
      targetDocument.defaultView.requestAnimationFrame(() => {
        targetDocument.defaultView.requestAnimationFrame(restoreStyleFrameOrder);
      });
    }
  };
  const appendMarked = (element, id) => {
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
  scheduleStyleFrameOrderRestore();
})();`
}
