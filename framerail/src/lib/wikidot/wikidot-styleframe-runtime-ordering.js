export const STYLEFRAME_ORDERING_RUNTIME_SOURCE = `  const head = targetDocument.head || targetDocument.documentElement;
  const markedStyleNodes = () => Array.from(
    head.querySelectorAll('[data-wikidot-style-frame="' + marker + '"]')
  );
  const generatedCssNodes = () => Array.from(
    head.querySelectorAll("[data-wikijump-generated-css]")
  );
  const generatedCssCloneNodes = () => Array.from(
    head.querySelectorAll("[data-wikijump-generated-css-clone]")
  );
  const stylePriority = (node) => {
    const value = Number.parseFloat(node.dataset.wikidotStylePriority || "");
    return Number.isFinite(value) ? value : 0;
  };
  const syncGeneratedCssClones = (markedNodes) => {
    const generatedNodes = generatedCssNodes();
    const existingClones = generatedCssCloneNodes();
    const clonesMatch =
      existingClones.length === generatedNodes.length &&
      existingClones.every(
        (clone, index) => clone.textContent === generatedNodes[index].textContent
      );
    if (markedNodes.length > 0 && clonesMatch) return existingClones;
    existingClones.forEach((node) => node.remove());
    if (markedNodes.length === 0) return [];
    return generatedNodes.map((source, index) => {
      const clone = targetDocument.createElement("style");
      clone.type = "text/css";
      clone.dataset.wikijumpGeneratedCssClone = String(index);
      clone.textContent = source.textContent;
      head.appendChild(clone);
      return clone;
    });
  };
  const restoreStyleFrameOrder = () => {
    const markedNodes = markedStyleNodes().sort(
      (left, right) => stylePriority(left) - stylePriority(right)
    );
    const cloneNodes = syncGeneratedCssClones(markedNodes);
    const desiredTail = [...markedNodes, ...cloneNodes];
    const currentNodes = Array.from(head.children);
    const tailOffset = currentNodes.length - desiredTail.length;
    const alreadyOrdered =
      tailOffset >= 0 &&
      desiredTail.every((node, index) => currentNodes[tailOffset + index] === node);
    if (!alreadyOrdered) {
      desiredTail.forEach((node) => head.appendChild(node));
    }
  };
  registry.restoreStyleFrameOrder = restoreStyleFrameOrder;
  const scheduleStyleFrameOrderRestore = () => {
    restoreStyleFrameOrder();
    setTimeout(restoreStyleFrameOrder, 0);
    setTimeout(restoreStyleFrameOrder, 250);
    if (typeof targetDocument.defaultView?.requestAnimationFrame === "function") {
      targetDocument.defaultView.requestAnimationFrame(() => {
        targetDocument.defaultView.requestAnimationFrame(restoreStyleFrameOrder);
      });
    }
  }`
