import type CSS from "csstype";

import OZONE from ".";

declare const YAHOO: any;
declare const fx: any;

/**
 * Backend for dialog boxes and hovertips.
 *
 * The shader (#odialog-shader )is the dark bit around the dialog box that
 * obscures the rest of the page.
 * The box container (#odialog-container) contains the dialog box (.owindow).
 */

// The hovertips code features elements with hovertip-related properties
// attached.
type HovertipOptions = {
  context?: string;
  text?: string;
  style?: CSS.Properties;
  noCursorHelp?: boolean;
  delay?: number;
  smartWidthLimit?: number;
  valign?: 'center';
};

export type HovertipElement = HTMLElement & {
  // TODO hovertip property should not be optional (?)
  hovertip?: HTMLElement;
  hovertipOptions?: HovertipOptions;
  hovertipEffect?: unknown;
};

// Storage for an effect (??)
const tempEffect: null | unknown = null;

function cleanAll (options?: { timeout: number }): void {
  /**
   * Clears both the shader and the box container.
   *
   * @param options: Time (ms) to wait before hiding
   */
  const timeout = options?.timeout ?? 200;

  setTimeout(() => factory.boxcontainer().hide(), timeout);
  setTimeout(() => factory.shader().hide(), timeout);
}

class Shader {
  /**
   * The shader that covers most of the screen during dialog presentation.
   */
  color: null | string;
  cssClass: null | string;

  constructor () {
    this.color = null;
    this.cssClass = null;
  }

  setColor (color: string): void {
    /**
     * ?? (unused)
     *
     * @param color: ??
     */
    this.color = color;
  }

  show (): void {
    /**
     * Shows a shader on the page, unless one already exists.
     */
    // If a shader element already exists, no need to create another
    if (document.getElementById("odialog-shader") !== null) {
      return;
    }
    // Otherwise, create a new one
    const shaderElement = document.createElement('div');
    shaderElement.id = 'odialog-shader';

    const body = document.getElementsByTagName('body')[0];
    const bodyHeight = OZONE.visuals.bodyHeight() + 50;
    const viewportHeight = YAHOO.util.Dom.getClientHeight();
    const height = Math.max(bodyHeight, viewportHeight);
    shaderElement.style.height = height + "px";

    if (this.color !== null) {
      shaderElement.style.backgroundColor = this.color;
    }
    if (this.cssClass === null) {
      shaderElement.className = "odialog-shader";
    } else {
      shaderElement.className = this.cssClass;
    }

    // extra iframe for stupid browsers
    // XXX Please scrap this
    if (window.navigator.userAgent.match(/msie/i)) {
      const shaderIframe = document.createElement('iframe');
      shaderIframe.id = "odialog-shader-iframe";
      shaderIframe.src = "/files--common/misc/blank.html";
      shaderIframe.frameBorder = "0";
      shaderIframe.className = 'odialog-shader-iframe';
      shaderIframe.style.height = height + "px";
      body.appendChild(shaderIframe);
    }
    body.appendChild(shaderElement);
  }

  hide (): void {
    /**
     * Hides the shader.
     */
    const body = document.getElementsByTagName('body')[0];
    const shaderElement = document.getElementById('odialog-shader');
    const shaderIframe = document.getElementById('odialog-shader-iframe');
    if (shaderElement !== null) {
      body.removeChild(shaderElement);
    }
    if (shaderIframe !== null) {
      body.removeChild(shaderIframe);
    }
  }
}

class BoxContainer {
  /**
   * Container for the dialog box.
   *
   * XXX Note that the Wikidot code requires throughout this class that both
   * concontainerElement and dialogElement are definitely assigned, but they
   * are not. Pending a refactor, for now I have used non-null assertions
   * wherever either of those values are required. Please bear in mind that
   * these assertions are not necessarily correct, but fixing them would be
   * outside the scope of this refactor.
   */
  containerElement: null | HTMLElement;
  dialogElement: null | HTMLElement;

  constructor () {
    this.containerElement = null;
    this.dialogElement = null;
    let el = document.getElementById("odialog-container");
    if (!el) {
      el = document.createElement('div');
      el.id = "odialog-container";
      const body = document.getElementsByTagName('body')[0];
      body.appendChild(el);
      this.containerElement = el;
    }
    el.style.display = "block";
  }

  setContent (content: string | HTMLElement): void {
    /**
     * Sets the content of the containerElement to either an HTML string or
     * an element.
     *
     * The pre-made dialogs in dialogs.ts will pass their own dialogElement
     * to this method during their show().
     *
     * @param content: The content of the dialog box.
     */
    this.clearContent();

    if (typeof content === 'string') {
      this.containerElement!.innerHTML = content;
    } else {
      this.containerElement!.appendChild(content);
    }

    OZONE.utils.formatDates(this.containerElement!);
    dialog.hovertip.dominit(this.containerElement!, { delay: 300 });

    this.dialogElement = this.containerElement!.getElementsByTagName('div')[0];
    this.dialogElement.style.visibility = "hidden";
    this.containerElement!.style.display = "block";
    // center by default
    this.centerContent();

    this.dialogElement.id = "owindow-1";

    // Iterate through all the divs in the dialog and do things to specific
    // ones
    // TODO Replace this with specific selectors, not a brute-force iteration
    const internalElements = this.dialogElement.getElementsByTagName('div');
    Array.from(internalElements).forEach(internalElement => {
      if (internalElement.className === "title") {
        internalElement.id = "ohandle-1";
        const dd1 = new YAHOO.util.DD(this.dialogElement!.id);
        dd1.setHandleElId(internalElement.id);
      }
      if (internalElement.className === "close") {
        YAHOO.util.Event.addListener(internalElement, "click", dialog.cleanAll);
      }
    });
  }

  clearContent (): void {
    /**
     * Destroy the dialog box.
     */
    this.dialogElement = null;
    this.containerElement!.innerHTML = "";
  }

  centerContent (): void {
    /**
     * Centre the dialog box in the middle of the page. XXX or screen?
     */
    const dialogElement = this.dialogElement;
    const height = dialogElement!.offsetHeight;
    const width = dialogElement!.offsetWidth;
    const vpHeight = YAHOO.util.Dom.getClientHeight();
    const vpWidth = YAHOO.util.Dom.getClientWidth();

    const posX = Math.max((vpWidth - width) * 0.5, 0);
    const posY = Math.max(
      OZONE.visuals.scrollOffsetY() + (vpHeight - height) * 0.5,
      0
    );

    YAHOO.util.Dom.setXY(dialogElement, [posX, posY]);
  }

  setContentObject (object: HTMLElement): void {
    /**
     * ?? (unused)
     */
    this.containerElement!.appendChild(object);
  }

  showContent (options?: { smooth: boolean }): void {
    /**
     * Show the content of the dialog box.
     */
    this.containerElement!.style.display = "block";

    if (options && options.smooth) {
      const ef = new fx.Opacity(this.dialogElement!, { duration: 300 });
      ef.setOpacity(0);
      this.dialogElement!.style.visibility = "visible";
      ef.custom(0, 1);
    } else {
      this.dialogElement!.style.visibility = "visible";
    }
  }

  hideContent (): void {
    /**
     * Hide the content of the dialog box.
     */
    this.dialogElement!.style.visibility = "hidden";
  }

  hide (options?: { smooth: boolean }): void {
    /**
     * ?? (doesn't seem to be used, but very hard to be sure)
     */
    if (options && options.smooth) {
      const ef = new fx.Opacity(this.dialogElement, { duration: 300 });
      ef.setOpacity(1);
      ef.custom(1, 0);
    }
    this.clearContent();
    document.getElementById("odialog-container")!.style.display = "none";
  }

  clickOutsideToHide (): void {
    /**
     * Attaches a listener to the shader that clears dialogs when it is
     * clicked.
     */
    YAHOO.util.Event.addListener("odialog-shader", "click", dialog.cleanAll);
  }

  changeContent (content: string | HTMLElement): void {
    /**
     * ?? (unused)
     *
     * @param content: The content to change the dialog to (presumably)
     */
    this.setContent(content);
    this.showContent();
  }
}

const factory = {
  /* The factory acts as a wrapper to ensure that no more than one shader or
   * dialog container can exist at the same time.
   */
  shader: function (): Shader {
    // Returns the current shader or generates a new one
    if (factory.stock.shader === null) {
      factory.stock.shader = new dialog.Shader();
    }
    return factory.stock.shader;
  },
  boxcontainer: function (): BoxContainer {
    // Returns the current dialog container or generates a new one
    if (factory.stock.boxcontainer === null) {
      factory.stock.boxcontainer = new dialog.BoxContainer();
    }
    return factory.stock.boxcontainer;
  },
  stock: {
    shader: null as null | Shader,
    boxcontainer: null as null | BoxContainer
  }
};

const hovertip = {
  container: null as null | unknown,
  bindings: [] as unknown[],

  init: function (): void {
    /**
     * Constructs the #odialog-hovertips element if it does not exist.
     */
    let el = document.getElementById('odialog-hovertips');
    if (!el) {
      el = document.createElement('div');
      el.id = 'odialog-hovertips';
      el.style.position = "absolute";
      el.style.zIndex = "100";
      el.style.top = "0";
      el.style.width = "100%";
      const body = document.getElementsByTagName('body')[0];
      body.appendChild(el);
      dialog.hovertip.container = el;
    }
  },

  makeTip: function (
    element: string | HTMLElement | HTMLCollection,
    options?: HovertipOptions
  ): void {
    /**
     * ??
     *
     * @param element: The element ??
     * Can be given as a string representing an element's ID, a reference to
     * an element itself, or an array of such references as returned by e.g.
     * getElementsByTagName().
     * @param options: Settings for the hovertip. May contain either context
     * or text, but should not contain both.
     *   context: The ID of the element to ??
     *   text: Text of the hovertip.
     *   style: Object containing style rules for the hovertip.
     *   noCursorHelp: ??
     *   delay: ??
     *   smartWidthLimit: ??
     *   valign: ??
     */
    // XXX What is 'el'? It's not the element representing the hovertip
    // itself - right? Is it the element that will show the hovertip once
    // hovered?
    // XXX What is 'tipEl'? Is that the hovertip?
    // TODO Rename those variables once we know

    // Type guard to check if the 'element' is a collection
    const isHtmlCollection = (
      element: HTMLElement | HTMLCollection
    ): element is HTMLCollection => 'length' in element;

    let el: HovertipElement;
    if (typeof element !== "string") {
      if (isHtmlCollection(element)) {
        // Recurse for each element in the array
        Array.from(element).forEach(element => {
          // XXX Type assertions like this are indicative of bad functions
          // TypeScript thinks `element` is type Element
          const htmlElement = element as HTMLElement;
          dialog.hovertip.makeTip(htmlElement, options);
        });
        return;
      } else {
        el = element;
      }
    } else {
      el = document.getElementById(element)!;
    }

    // options can be: text (text) or context element id (context)
    dialog.hovertip.init(); // just for sure
    if (!el) {
      return;
    }
    if (el.hovertip) {
      // XXX What is el.hovertip? Why is it needed? Where is it assigned?
      return;
    }

    let tipEl: HTMLElement;
    if (options && options.context) {
      // Context contains an existing ID to use as tipEl
      const el = document.getElementById(options.context);
      if (el === null) {
        // XXX This should probably throw an error
        return;
      }
      tipEl = el;
    } else if (options && options.text) {
      // create a new div
      tipEl = document.createElement('div');
      tipEl.innerHTML = `<div class="content">${options.text}</div>`;
    } else {
      // If no context or text is given, use text from the element's title
      // attribute
      let title: string | null = null;
      if (el.attributes) {
        // TODO Replace this with something that selects the title
        // specifically without brute-force looping
        for (let x = 0; x < el.attributes.length; x++) {
          if (el.attributes[x].nodeName.toLowerCase() === 'title') {
            title = el.attributes[x].nodeValue;
            el.attributes[x].nodeValue = '';
          }
        }
      }
      if (title === null) {
        // XXX Should probably throw an error
        return;
      }
      tipEl = document.createElement('div');
      tipEl.innerHTML = `<div class="content">${title}</div>`;
    }

    tipEl.classList.add("hovertip");
    if (options) {
      el.hovertipOptions = options;
    }
    if (options && 'style' in options) {
      Object.assign(tipEl.style, options.style);
    }

    // fix if not "content" div inside.
    const subDivs = tipEl.getElementsByTagName('div');
    let hasContent = false;
    for (let i = 0; i < subDivs.length; i++) {
      if (YAHOO.util.Dom.hasClass(subDivs[i], 'content')) {
        hasContent = true;
      }
    }
    if (!hasContent) {
      tipEl.innerHTML = `<div class="content">${tipEl.innerHTML}</div>`;
    }

    // make sure some properties are set properly
    el.hovertip = tipEl;
    const eff = new fx.Opacity(el.hovertip, { duration: 300 });
    el.hovertipEffect = eff;

    tipEl.style.position = "absolute";
    tipEl.style.display = "none";

    // for debugging
    tipEl.style.border = "1px solid black";

    if (
      el.tagName !== 'A' &&
      (!options || !options.noCursorHelp)
    ) {
      el.style.cursor = "help";
    }
    // moving along...
    document.getElementById('odialog-hovertips')!.appendChild(tipEl);

    // somehow make a binding now
    dialog.hovertip.bindings.push([el, tipEl]);

    YAHOO.util.Event.addListener(el, "mousemove", dialog.hovertip._mousemove);
    YAHOO.util.Event.addListener(el, "mouseout", dialog.hovertip._mouseout);
    YAHOO.util.Event.addListener(el, "mouseover", dialog.hovertip._mouseover);
  },

  _mouseover: function (e: MouseEvent): void {
    /**
     * Callback placed on elements that should show a hovertip when hovered.
     *
     * Shows the hovertip.
     */
    const el: HovertipElement = YAHOO.util.Event.getTarget(e);

    const tipEl = el.hovertip!;
    tipEl.style.visibility = "hidden";
    tipEl.style.opacity = "0";
    tipEl.style.display = "block";
    const options = el.hovertipOptions;
    const eff = el.hovertipEffect;
    // position to (0,0) to avoid glitches
    YAHOO.util.Dom.setXY(el.hovertip, [0, 0]);
    // trigger mousemove too!
    dialog.hovertip._mousemove(e);

    if (options && options.delay) {
      dialog.tempEffect = eff;

      setTimeout(() => {
        // @ts-expect-error Need new animation library
        if (OZONE.dialog.tempEffect.el.style.opacity === 0) {
          // @ts-expect-error Need new animation library
          OZONE.dialog.tempEffect.custom(0, 1);
        }
      }, options.delay);
    } else {
      // @ts-expect-error Need new animation library
      eff.custom(0, 1);
    }
  },

  _mousemove: function (e: MouseEvent): void {
    /**
     * Callback placed on elements that should show a hovertip when hovered.
     *
     * Moves the hovertip with the mouse.
     */
    const el: HovertipElement = YAHOO.util.Event.getTarget(e);

    const tipEl = el.hovertip!;

    // position and display the tip

    // get mouse position
    let posx = 0;
    let posy = 0;
    // XXX I am pretty sure the below snippet shouldn't happen, so I have
    // commented it out
    // if (!e) {
    //   e = window.event;
    // }
    if (e.pageX || e.pageY) {
      posx = e.pageX;
      posy = e.pageY;
    } else if (e.clientX || e.clientY) {
      posx = e.clientX + document.documentElement.scrollLeft;
      posy = e.clientY + document.documentElement.scrollTop;
    }
    // position the tipEl

    // now calculate where to position the tip box

    // get viewport size
    const vHeight = YAHOO.util.Dom.getViewportHeight();
    const vWidth = YAHOO.util.Dom.getViewportWidth();
    const tipElHeight = tipEl.offsetHeight;
    const tipElWidth = tipEl.offsetWidth;

    const border = 20; // border (whitearea) size

    if (el.hovertipOptions && el.hovertipOptions.smartWidthLimit) {
      const vlimit = el.hovertipOptions.smartWidthLimit;
      if (tipElWidth > vlimit * vWidth) {
        tipEl.style.width = vlimit * vWidth + 'px';
      }
    }

    // not to go outsite right/bottom border
    // assume sizes are considerably smaller than the
    // viewport size!
    if (el.hovertipOptions && el.hovertipOptions.valign) {
      if (el.hovertipOptions.valign === 'center') {
        if (
          (vHeight - e.clientY) < (tipElHeight + 2 * border) &&
          e.clientY > (tipElHeight + 1.5 * border)
        ) {
          posy -= tipElHeight + 1.5 * border;
        }
        posy += border;
        posx = e.clientX - tipElWidth * 0.5;
        if (posx + tipElWidth > vWidth - border) {
          posx = vWidth - tipElWidth - border;
        }
        if (posx < border) {
          posx = border;
        }
      }
    } else {
      if (
        (vWidth - e.clientX) < (tipElWidth + 2 * border) &&
        e.clientX > (tipElWidth + 1.5 * border)
      ) {
        posx -= tipElWidth + 1.5 * border;
      }
      if (
        (vHeight - e.clientY) < (tipElHeight + 2 * border) &&
        e.clientY > (tipElHeight + 1.5 * border)
      ) {
        // XXX This is the same check and effect as the one in the valign
        // block - refactoring should consolidate these
        posy -= tipElHeight + 1.5 * border;
      }
      posx += border;
      posy += border;
    }
    YAHOO.util.Dom.setXY(tipEl, [posx, posy]);
  },

  _mouseout: function (e: MouseEvent): void {
    /**
     * Callback placed on elements that should show a hovertip when hovered.
     *
     * Hides the hovertip.
     */
    // just hide it!
    const el: HovertipElement = YAHOO.util.Event.getTarget(e);

    const tipEl = el.hovertip!;
    tipEl.style.display = "none";
  },

  dominit: function (
    topElementOrId: string | HTMLElement,
    options?: HovertipOptions
  ): void {
    /**
     * Starting from the given element, search its children recursively for
     * pairs of divs with IDs #{} and #{}-hovertip. Store the binding
     * between the two elements.
     *
     * @param topElementOrId: The ID of the element to be used as the
     * starting point for searching, or a reference to that element.
     * @param options: ??
     */
    dialog.hovertip.init(); // just for sure
    let topElement: HTMLElement | null;
    if (typeof topElementOrId === 'string') {
      topElement = document.getElementById(topElementOrId);
    } else {
      topElement = topElementOrId;
    }
    // If the top element doesn't exist... just search the whole document
    // instead?? XXX
    if (topElement === null) {
      topElement = document.documentElement;
    }
    const allDivs = topElement.getElementsByTagName('div');
    Array.from(allDivs).filter(element => {
      return element.id.endsWith("-hovertip");
    }).forEach(tipEl => {
      const elId = tipEl.id.replace("-hovertip", "");
      const el = document.getElementById(elId);
      if (el) {
        if (!options) {
          options = {};
        }
        options.context = tipEl.id;
        dialog.hovertip.makeTip(el, options);
      }
    });
  },

  hideAll: function (): void {
    /**
     * Hides every hovertip.
     */
    Array.from(
      document.getElementById('odialog-hovertips')!
        .getElementsByClassName('hovertip') as HTMLCollectionOf<HTMLElement>
    ).forEach(hovertip => {
      hovertip.style.display = 'none';
    });
  }
};

export const dialog = {
  tempEffect,
  cleanAll,
  factory,
  Shader,
  BoxContainer,
  hovertip
};
