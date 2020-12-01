import { stringify } from "query-string";

import OZONE from ".";
import { ogettext } from "./loc";
import { HovertipElement } from "./dialog";

declare const YAHOO: any;

export const utils = {
  // The time of the most recent JS lock. 0 means no lock currently set.
  _javascriptLoadLock: 0,

  formToArray: function (formId: string): Record<string, string> {
    /**
     * Takes a form on the page by ID and returns its data as an object.
     *
     * TODO Needs reworking or replacing.
     *
     * @param formID: The ID of the form to get.
     */
    const form = document.getElementById(formId);
    if (form === null) {
      // XXX Throw error?
      return {};
    }

    // Type guard functions to narrow down what the element is
    const isInput = (el: Element): el is HTMLInputElement => {
      return el.tagName === 'INPUT';
    };
    const isTextarea = (el: Element): el is HTMLTextAreaElement => {
      return el.tagName === 'TEXTAREA';
    };

    // process different form elements (traverse)
    const values: { [name: string]: string } = {};
    Array.from(form.children).forEach(element => {
      if (isTextarea(element)) {
        values[element.name] = element.value;
      }
      if (isInput(element)) {
        const type = element.getAttribute('type');
        if (type === 'text' || type === 'hidden' || type === 'password') {
          values[element.name] = element.value;
        }
        if (type === 'checkbox' && element.checked === true) {
          values[element.name] = 'on';
        }
        if (type === 'radio' && element.checked === true) {
          values[element.name] = element.value;
        }
      }
    });
    return values;
  },

  arrayToPostData: function (
    data: { [parameter: string]: string | number | undefined | null | boolean } | null
  ): string | null {
    /**
     * Generates a URL query string from an object.
     *
     * @param data: The object from which to make a string.
     */
    if (data === null) {
      return null;
    }
    return stringify(data);
  },

  addJavascriptUrl: function (
    url: string,
    onLoadCallback?: () => void,
    noReload = false
  ): void {
    /**
     * Attaches a JS script by URL to the page head.
     *
     * @param url: The URL of the script to attach.
     * @param onLoadCallback: A callback to fire once the script has been
     * attached to the page (unused)
     * @param noReload: If the script is already present on the page, ignore
     * it; otherwise, remove and re-add it (unused)
     */
    // If there is a JS loading lock, wait until it has expired plus two
    // seconds
    if (
      utils._javascriptLoadLock &&
      (new Date()).getTime() < utils._javascriptLoadLock + 2000
    ) {
      setTimeout(
        () => utils.addJavascriptUrl(url, onLoadCallback, noReload), 50
      );
      return;
    }

    // Unset the lock
    utils._javascriptLoadLock = 0;

    // Check if this script is already present
    const head = document.getElementsByTagName('head')[0];
    Array.from(head.getElementsByTagName('script')).forEach(script => {
      if (script.getAttribute('src') === url) {
        // If we have been asked not to reload the script, do nothing
        if (noReload) {
          if (onLoadCallback) {
            onLoadCallback();
          }
          return;
        }
        // Otherwise, if the script is present, remove it and proceed
        head.removeChild(script);
      }
    });

    // Add the script to the page
    utils._javascriptLoadLock = (new Date()).getTime();
    const newScriptElement = document.createElement('script');
    newScriptElement.setAttribute('type', 'text/javascript');
    newScriptElement.setAttribute('src', url);

    // When the new element has finished loading, unset the lock and fire the
    // callback
    YAHOO.util.Event.addListener(newScriptElement, 'load', function () {
      utils._javascriptLoadLock = 0;
      if (onLoadCallback) {
        onLoadCallback();
      }
    });
    head.appendChild(newScriptElement);
  },

  addStyleUrl: function (
    url: string,
    onLoadCallback?: () => void,
    noReload = false
  ): void {
    /**
     * Attaches a CSS stylesheet by URL to the page head.
     *
     * @param url: The URL of the stylesheet to attach.
     * @param onLoadCallback: A callback to fire once the stylesheet has been
     * attached to the page (unused)
     * @param noReload: If the script is already present on the page, ignore
     * it; otherwise, remove and re-add it (unused)
     */
    const head = document.getElementsByTagName('head')[0];
    Array.from(head.getElementsByTagName('link')).forEach(link => {
      if (
        link.type === 'text/css' &&
        link.getAttribute('src') === url
      ) {
        // If we have been asked not to reload the script, do nothing
        if (noReload) {
          if (onLoadCallback) {
            onLoadCallback();
          }
          return;
        }
        // Otherwise, if the script is present, remove it and proceed
        head.removeChild(link);
      }
    });

    // Add the script to the page
    const newScriptElement = document.createElement('link');
    newScriptElement.rel = 'stylesheet';
    newScriptElement.type = 'text/css';
    newScriptElement.href = url;
    //
    // When the new element has finished loading, fire the callback
    if (onLoadCallback) {
      YAHOO.util.Event.addListener(newScriptElement, 'load', onLoadCallback);
    }
    head.appendChild(newScriptElement);
  },

  setInnerHTMLContent: function (elementId: string, content: string): void {
    /**
     * Sets the InnerHTMLContent of an element.
     *
     * @param elementId: The ID of the element to change.
     * @param content: The desired HTML string.
     */
    const element = document.getElementById(elementId);
    if (element !== null) {
      element.innerHTML = content;
      utils.formatDates(elementId);
      OZONE.dialog.hovertip.dominit(element);
    }
  },

  disableEnterKey: function (e: KeyboardEvent): boolean {
    /**
     * ??
     *
     * @param e: A DOM event.
     */
    // disable for textareas!
    if ((e.target as Element).tagName === 'TEXTAREA') {
      return true;
    }

    return e.code !== 'Enter';
  },

  escapeHtml: function (htmlText: string): string {
    /**
     * Replaces some HTML-special characters with text.
     *
     * @param htmlText: The HTML to escape.
     */
    return htmlText
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;');
  },

  unescapeHtml: function (htmlText: string): string {
    /**
     * Unreplaces some HTML-special characters with text.
     *
     * @param htmlText: The HTML to unescape.
     */
    return htmlText
      .replace(/&amp;/g, '&')
      .replace(/&lt;/g, '<')
      .replace(/&gt;/g, '>');
  },

  formatDates: function (topElementOrId?: string | HTMLElement): void {
    /**
     * Within the element of ID topElementId, format all spans with class
     * 'odate' to contain text representing the date.
     *
     * The date is taken from the inner text of the class, which should be a
     * UNIX timestamp.
     *
     * TODO The bulk of this code can probably be outsourced. Most npm
     * date-formatting modules do not appear to use percent-encoded dates. It
     * would be nice to deprecate that, if possible.
     *
     * @param topElementOrId: The element whose children should be
     * searched for span.odate, or its ID. If not provided, the whole document
     * is searched.
     */
    const monthNames = [
      'January', 'February', 'March', 'April', 'May', 'June',
      'July', 'August', 'September', 'October', 'November', 'December'
    ];
    const monthNamesShort = [
      'Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun',
      'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'
    ];
    const dayNames = [
      'Sunday', 'Monday', 'Tuesday', 'Wednesday',
      'Thursday', 'Friday', 'Saturday'
    ];
    const dayNamesShort = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];

    let dateElements: NodeListOf<HTMLSpanElement>;
    if (topElementOrId === undefined) {
      dateElements = document.querySelectorAll<HTMLSpanElement>('span.odate');
    } else {
        if (typeof topElementOrId === 'string') {
          const topElement = document.getElementById(topElementOrId);
          if (topElement === null) {
            // TODO Throw error
            return;
          }
          dateElements = topElement.querySelectorAll<HTMLSpanElement>('span.odate');
        } else {
          dateElements = topElementOrId.querySelectorAll<HTMLSpanElement>('span.odate');
        }
    }

    (Array.from(dateElements) as HTMLElement[]).forEach(dateElement => {
      let dateString = "";
      // The inner text contains timestamp to replace
      const inner = dateElement.innerHTML;
      // If the inner text is just a number, that is the timestamp
      if (inner.match(/^[0-9]+$/)) {
        const timestamp = parseInt(inner);
        const date = new Date();
        date.setTime(timestamp * 1000);
        dateString = date.toLocaleString();
      }
      // If the text contains a bar, formatting arguments follow
      if (inner.match(/^[0-9]+\s*\|.*$/)) {
        // TODO Refactor this bit
        const timestamp = parseInt(inner.replace(/^([0-9]+)\s*\|.*/, '$1'));
        const format = inner.replace(/^[0-9]+\s*\|\s*(.*?)(?:\|(.*))?$/, '$1');
        const options = inner.replace(/^[0-9]+\s*\|\s*(.*?)(?:\|(.*))?$/, '$2');

        const date = new Date();
        date.setTime(timestamp * 1000);

        const year = date.getFullYear();
        const month = date.getMonth();
        const dayOfMonth = date.getDate();
        const hours = date.getHours();
        const hours12 = ((date.getHours() - 1) % 12 + 1);
        const minute = date.getMinutes();
        const seconds = date.getSeconds();

        dateString = format
          // Shortcuts for preset abbreviation sets
          .replace(/%r/g, '%I:%M:%S %p')
          .replace(/%R/g, '%H:%M')
          .replace(/%T/g, '%H:%M:%S')
          .replace(/%D/g, '%m/%d/%y')

          // %a for abbreviated weekday name
          .replace(/%a/g, dayNamesShort[date.getDay()])
          // %A for full weekday name
          .replace(/%A/g, dayNames[date.getDay()])
          // %b for short month name
          .replace(/%b/g, monthNamesShort[date.getMonth()])
          // %B for full month name
          .replace(/%B/g, monthNames[date.getMonth()])
          // %c for local date representation
          .replace(/%c/g, date.toLocaleString())
          // %d for zero-filled day of the month
          .replace(/%d/g, `${dayOfMonth}`.padStart(2, "0"))
          // %e for day of the month
          .replace(/%e/g, `${dayOfMonth}`)
          // %H for hour 00-23
          .replace(/%H/g, `${hours}`.padStart(2, "0"))
          // %I for hour 00-12
          .replace(/%I/g, `${hours12}`.padStart(2, "0"))
          .replace(/%m/g, `${month + 1}`.padStart(2, "0"))
          // %M for minutes
          .replace(/%M/g, `${minute}`.padStart(2, "0"))
          .replace(/%p/g, (hours < 12) ? 'AM' : 'PM')

          .replace(/%S/g, `${seconds}`.padStart(2, "0"))
          .replace(/%y/g, `${year}`.substring(2))
          .replace(/%Y/g, `${year}`);

        if (dateString.match(/%z/i)) {
          // try to get zone from locale date string
          let zone = date.toLocaleString().replace(/^.*?([A-Z]{3,}(?:\+[0-9]+)?).*$/, '$1');
          if (zone === date.toLocaleString()) {
            // If that failed, then just get the timezone offset and display
            // that instead
            let zoneoffset = date.getTimezoneOffset();
            zoneoffset = -zoneoffset / 60;
            zone = `${zoneoffset > 0 ? "+" : "-"}${zoneoffset < 10 ? "0" : ""}${zoneoffset}`;
          }
          dateString = dateString.replace(/%z/ig, zone);
        }
        if (dateString.match(/%O/) || options.match(/agohover/)) {
          // time ago
          let secAgo = OZONE.request.timestamp - timestamp;
          secAgo += Math.floor(
            ((new Date()).getTime() - OZONE.request.date.getTime()) / 1000
          );
          const agoString = OZONE.utils.calculateDateAgo(secAgo);

          dateString = dateString.replace(/%O/, agoString);
          if (options.match(/agohover/)) {
            // TODO Localisation
            const hovertext = `${agoString} ago`;
            OZONE.dialog.hovertip.makeTip(
              dateElement, { text: hovertext, style: { width: 'auto' } }
            );
            // dateElement is now a HovertipElement
            YAHOO.util.Event.addListener(dateElement, 'mouseover', function (
              this: HovertipElement,
              _event: Event
            ) {
              // Yahoo sets this to the scope element (dateElement)
              if (this.hovertip) {
                // XXX Shouldn't be necessary when hovertip isn't optional
                this.hovertip.getElementsByTagName('div')[0].innerHTML = `${agoString} ${ogettext('ago')}`;
              }
            });
          }
        }
      }
      if (dateString) {
        dateElement.innerHTML = dateString;
        dateElement.style.visibility = 'visible';
        dateElement.style.display = 'inline';
      }
    });
  },

  calculateDateAgo: function (secAgo: number): string {
    /**
     * Generates a string representing how long ago a time was from its delta
     * from now in seconds.
     *
     * @param secAgo: The number of seconds ago that the time was.
     */
    let agoString: string;
    if (secAgo >= 60 * 60 * 24) {
      const days = Math.floor(secAgo / (60 * 60 * 24));
      agoString = '' + days + ' ' + ((days) > 1 ? ogettext('days') : ogettext('day'));
    } else if (secAgo >= 60 * 60) {
      const hours = Math.floor(secAgo / (60 * 60));
      agoString = '' + hours + ' ' + ((hours) > 1 ? ogettext('hours') : ogettext('hour'));
    } else if (secAgo >= 60) {
      const minutes = Math.floor(secAgo / 60);
      agoString = '' + minutes + ' ' + ((minutes) > 1 ? ogettext('minutes') : ogettext('minute'));
    } else {
      if (secAgo === 0) { secAgo++; }
      agoString = '' + secAgo + ' ' + ((secAgo) > 1 ? ogettext('seconds') : ogettext('second'));
    }
    return agoString;
  },

  loadPage: function (url: string, parameters: Record<string, string>): void {
    /**
     * ?? (unused)
     *
     * Wikidot comment:
     * This is tricky. Loads desired url with parameters but the parameters
     * are contained in the POST body.
     *
     * @param url: ??
     * @param parameters: ??
     */
    // create a dummy form
    const form = document.createElement('form');
    for (const p in parameters) {
      const input = document.createElement('input');
      input.type = 'hidden';
      input.name = p;
      input.value = parameters[p];
      form.appendChild(input);
    }
    form.name = 'loadPageForm';
    form.action = url;
    form.method = 'post';
    form.display = 'none';
    form.target = '_self';
    document.getElementsByTagName('body')[0].appendChild(form);
    form.submit();
  },

  getCookie: function (cookieName: string): string | null {
    /**
     * Gets the contents of a cookie by name, or null if not present.
     *
     * @param cookieName: The name of the cookie to get.
     */
    if (document.cookie.length > 0) {
      let cStart = document.cookie.indexOf(cookieName + '=');
      if (cStart !== -1) {
        cStart = cStart + cookieName.length + 1;
        let cEnd = document.cookie.indexOf(';', cStart);
        if (cEnd === -1) {
          cEnd = document.cookie.length;
        }
        return unescape(document.cookie.substring(cStart, cEnd));
      }
    }
    return null;
  },

  olang: function (text: string): string {
    /**
     * ??
     * Seems to be used for rudimentary localisation.
     * That's at least three competing localisation systems I've seen now.
     * XXX Unify them
     *
     * Filter all substrings of form e.g. [[olang en:sdadsd|pl:asdadad|]]
     *
     * @param text: ??
     */
    return text.replace(/\[\[olang (.*?)\|\]\]/g, function (str) {
      const lang = OZONE.lang;
      const lr = new RegExp(lang + ':([^|]*)(\\||]])');
      const res = str.match(lr);
      if (res) {
        return res[1];
      }
      return str;
    });
  }
};
