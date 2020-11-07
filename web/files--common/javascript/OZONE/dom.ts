import OZONE from ".";

export const dom = {
  insertAfter: function (
    parentNode: Node,
    node: Node,
    referenceNode: Node
  ): void {
    /**
     * Inserts a new node into the document, placed after a reference node.
     *
     * @param parentNode: The node that contains the referenceNode, and that
     * will contain the new node.
     * @param node: The new node to add.
     * @param referenceNode: The node to insert the new node after.
     */
    if (referenceNode.nextSibling) {
      parentNode.insertBefore(node, referenceNode.nextSibling);
    } else {
      parentNode.appendChild(node);
    }
  },

  onDomReadyCallbacks: [] as (() => void)[],

  onDomReady: function (
    callback: number | (() => void),
    el: string,
    doc?: Document
  ): void {
    /**
     * Executes the callback when the document is ready.
     *
     * TODO Can almost certainly be outsourced.
     *
     * So far as I can tell, el is always "dummy-ondomready-block", which is
     * an empty (dummy) block used as a canary for checking that the page has
     * loaded, and doc is unused.
     *
     * @param callback: The callback, OR a number referring to the callback's
     * index in the stored callbacks array. TODO Refactor this.
     * @param el: The ID of an element to look for to check that the page has
     * been loaded.
     * @param doc: A replacement for the document object.
     */
    if (!doc) {
      doc = document;
    }
    if (
      typeof doc.getElementsByTagName !== 'undefined' &&
      (doc.getElementsByTagName('body')[0] !== null || doc.body !== null) &&
      (typeof el !== 'string' || doc.getElementById(el) !== null)
    ) {
      if (typeof callback === 'function') {
        callback();
      } else {
        OZONE.dom.onDomReadyCallbacks[callback]();
      }
    } else {
      let fid: number;
      if (typeof callback === 'function') {
        fid = OZONE.dom.onDomReadyCallbacks.push(callback) - 1;
      } else {
        fid = callback;
      }

      // Wait and try again
      setTimeout(() => OZONE.dom.onDomReady(fid, el), 200);
    }
  }
};
