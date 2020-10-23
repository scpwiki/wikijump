export const visuals = {
  cursorWait: function (): void {
    /**
     * Adds the wait class to body, which changes the cursor to 'wait'.
     */
    const body = document.getElementsByTagName('body')[0];
    YAHOO.util.Dom.addClass(body, 'wait');
  },

  cursorClear: function (): void {
    /**
     * Removes the wait class from body.
     */
    const body = document.getElementsByTagName('body')[0];
    YAHOO.util.Dom.removeClass(body, 'wait');
  },

  scrollTo: function (
    elementId: string,
    options?: {
      blink?: boolean
      alterHref?: boolean
    }
  ): void {
    /**
     * @alias scrollToCenter
     */
    OZONE.visuals.scrollToCenter(elementId, options);
  },

  scrollToCenter: function (
    elementId: string,
    options?: {
      blink?: boolean
      alterHref?: boolean
    }
  ): void {
    /**
     * Scroll to the element such that it is in the centre of the page.
     *
     * @param elementId: The ID of the element to scroll to.
     * @param options: ??
     */
    const scrollEffect = new fx.ScrollCenter({ duration: 200, transition: fx.sineOut });
    scrollEffect.scrollTo(elementId);
    if (options != null && options.blink === true) {
      const ofx34 = new fx.Opacity(elementId, { duration: 150, transition: fx.circ });
      setTimeout(() => ofx34.custom(1, 0.1), 300);
      setTimeout(() => ofx34.custom(0.1, 1), 1000);
    }
    if (options != null && options.alterHref === true) {
      // commented out because causes page jump...
    }
  },

  scrollOffsetY: function (): number {
    /**
     * Gets the vertical scroll offset.
     */
    return window.scrollY
  },

  bodyHeight: function (): number {
    /**
     * Gets the height of the document.
     */
    const test1 = document.body.scrollHeight;
    const test2 = document.body.offsetHeight;
    if (test1 > test2) {
      // all but Explorer Mac
      return document.body.scrollHeight;
    } else {
      // Explorer Mac;
      // would also work in Explorer 6 Strict, Mozilla and Safari
      return document.body.offsetHeight;
    }
  },

  initScroll: function (): void {
    /**
     * If the URL contains a target, scroll to it.
     * XXX I think browsers do that themselves these days?
     */
    if (window.location.hash != null && window.location.href !== '') {
      const id = window.location.hash.replace(/#/, '');
      if (id != null && id !== '' && $(id)) {
        OZONE.visuals.scrollTo(id, { blink: true });
      }
    }
  },

  /** TODO later. */
  highlightText: function (rootElementId: string, text: string): void {
    /**
     * ??
     *
     * @param rootElementId: The ID of the element to ??
     * @param text: ??
     */
    // split the text by space (if any)
    if (text.indexOf(' ') !== -1) {
      const tarray = text.split(/ +/);
      tarray.forEach(t => {
        if (!t.match(/^-/)) {
          OZONE.visuals.highlightText(rootElementId, t);
        }
      });
      return;
    }

    const rootElement = document.getElementById(rootElementId);
    if (rootElement === null) {
      return;
    }

    // recurrence first
    if (rootElement.hasChildNodes) {
      Array.from(rootElement.childNodes).forEach(childNode => {
        // XXX The old $ function had pass-through for objects so this no
        // longer works - needs a workaround
        OZONE.visuals.highlightText(childNode, text);
      });
    }
    if (rootElement.nodeType === 3) { // text node
      // purify text a bit

      const reg = new RegExp(text, 'gi');
      if (rootElement.nodeValue.match(reg)) {
        const contArray = (' ' + rootElement.nodeValue + ' ').split(reg);
        const p = rootElement.parentNode;
        for (let i = 0; i < contArray.length; i++) {
          if (i !== 0) {
            const span = document.createElement('span');
            span.className = 'search-highlight';
            span.appendChild(document.createTextNode(text));
            p.insertBefore(span, rootElement);
          }
          const z = document.createTextNode(contArray[i]);
          if (i !== contArray.length - 1) {
            p.insertBefore(z, rootElement);
          } else {
            p.replaceChild(z, rootElement);
          }
        }
      }
    }
  }
};
