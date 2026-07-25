export const STYLEFRAME_ORDERING_RUNTIME_SOURCE = `  const head = targetDocument.head || targetDocument.documentElement;
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
  }`
