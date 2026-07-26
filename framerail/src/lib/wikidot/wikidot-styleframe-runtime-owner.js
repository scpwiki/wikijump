import {
  safeStyleFrameScriptJson,
  STYLEFRAME_REGISTRY
} from "./wikidot-styleframe-contract.js"

export const STYLEFRAME_OWNER_RUNTIME_SOURCE = `  const targetWindow = window.parent && window.parent !== window
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
    registry.restoreStyleFrameOrder?.();
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
  }`
