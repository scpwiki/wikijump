import { stringify } from "query-string";

/* global YAHOO, fx */

const ogettext = function (mid: string): string {
  return OZONE.loc.getMessage(mid, OZONE.lang);
};

type RequestModuleParameters = {
  action?: 'FileAction'
  event?: 'checkFileExists'
  moduleName?: string
  pageId?: number
  callbackIndex?: number
  wikijump_token7?: string
}

type RequestModuleOptions = {
  clearRequestQueue: boolean
}

// XXX Temporary until we deprecate Yahoo UI
type YahooResponse = {
  status: number
  statusText: string
  responseText: string
}

const OZONE = {

  ajax: {

    _callbackArray: [] as unknown[],
    _callbackArrayIndex: 0,

    // The time of the most recent JS lock, or false if there is none.
    // TODO Make this just a straight-up number.
    _javascriptLoadLock: false,

    /**
     * arg - extra parameter passed to the callback as a second parameter
     */
    requestModule: function (
      moduleName: string,
      parameters: RequestModuleParameters,
      callback: (parameters: unknown) => unknown,
      arg: unknown,
      options: RequestModuleOptions
    ): unknown {
      /**
       * ??
       *
       * @param moduleName: The name of the module being requested, or null if
       * some condition (??)
       * @param parameters: Parameters to pass to the callback.
       * @param callback: ??
       * @param arg: Extra parameter passed to the callback
       * @param options: ??
       */
      OZONE.visuals.cursorWait();
      if (parameters === null) {
        parameters = {};
      }
      if (moduleName === null || moduleName === '') {
        moduleName = 'Empty';
      }
      parameters.moduleName = moduleName;

      if (options && options.clearRequestQueue) {
        OZONE.ajax._callbackArray = [];
      }

      // TODO The callbackIndex can probably be refactored out
      const callbackIndex = OZONE.ajax._callbackArrayIndex++;
      OZONE.ajax._callbackArray[callbackIndex] = {
        callback: callback,
        arg: arg
      };

      parameters.callbackIndex = callbackIndex;

      // add token information
      const token = OZONE.utils.getCookie('wikijump_token7');
      if (token === null) {
        alert('Error processing the request.\n\nYou have no valid security token which is required to prevent identity theft.\nPlease enable cookies in your browser if you have this option disabled and reload the page.');
        OZONE.visuals.cursorClear();
        return;
      }
      parameters.wikijump_token7 = token;

      YAHOO.util.Connect.asyncRequest(
        'POST',
        '/ajax-module-connector.php',
        OZONE.ajax.requestModuleCallback,
        OZONE.utils.arrayToPostData(parameters)
      );
    },

    requestModuleCallback: {
      success: function (responseObject: YahooResponse): void {
        // Response comes from yahooui/connection.js/createResponseObject
        // TODO What is the type of the returned JSON?
        const response = JSON.parse(responseObject.responseText);

        if (response.status === 'wrong_token7') {
          // TODO: De-Wikijump.com-ize - change
          alert("Wikijump security error:\n\nYour authentication token in the request is not valid. Please enable cookies in your browser and try to repeat the action.\n\nIf you see this message on the page not associated with the Wikijump wiki hosting it probably means an identity theft attempt or improper use of Wikijump service.");
          OZONE.visuals.cursorClear();
          return;
        }

        const callbackIndex = response.callbackIndex;
        if (callbackIndex === null) {
          OZONE.visuals.cursorClear();
          OZONE.dialog.cleanAll();
        }
        if (!OZONE.ajax._callbackArray[callbackIndex]) {
          return;
        }
        const callback = OZONE.ajax._callbackArray[callbackIndex].callback;
        if (!callback) {
          alert('internal: callback error');
        }
        const arg = OZONE.ajax._callbackArray[callbackIndex].arg;
        // call callback

        if (arg != null) {
          callback(response, arg);
        } else {
          callback(response);
        }

        // attach javascript (if any)
        if (response.jsInclude != null) {
          response.jsInclude.forEach(jsUrl => {
            OZONE.utils.addJavascriptUrl(jsUrl);
          });
        }
        if (response.cssInclude != null) {
          response.cssInclude.forEach(cssUrl => {
            OZONE.utils.addStyleUrl(cssUrl);
          });
        }
        OZONE.visuals.cursorClear();
      },
      failure: function (responseObject: YahooResponse): void {
        alert('The ajax request failed. Please check your internet connection or\n' +
          'report a bug if the error repeats during your work.' + '\ncode:' + responseObject.status);

        OZONE.visuals.cursorClear();
        OZONE.dialog.cleanAll();
      }

    },

    requestQuickModule: function (
      moduleName: unknown,
      parameters: unknown,
      callback: unknown
    ): void {
      /**
       * ?? (unused)
       *
       * @param moduleName: ??
       * @param parameters: ??
       * @param callback: ??
       */
      if (parameters === null) {
        parameters = {};
      }
      if (moduleName === null || moduleName === '') {
        alert('Quick module name empty.');
      }

      const callbackIndex = OZONE.ajax._callbackArrayIndex++;
      OZONE.ajax._callbackArray[callbackIndex] = callback;

      parameters.callbackIndex = callbackIndex;

      const postdata = JSON.stringify(parameters);
      const internalCallback = OZONE.ajax.requestQuickModuleCallback;
      YAHOO.util.Connect.asyncRequest('POST', '/quickmodule.php?module=' + moduleName, internalCallback, postdata);
    },

    requestQuickModuleCallback: {
      /**
       * ?? (unused)
       */
      success: function (responseObject: YahooResponse): void {
        // process response
        const response = JSON.parse(responseObject.responseText);
        const callbackIndex = response.callbackIndex;
        const callback = OZONE.ajax._callbackArray[callbackIndex];
        callback(response);
      },
      failure: function (_responseObject: YahooResponse): void {
        alert('The ajax request failed. Please check your internet connection or\nreport a bug if the error repeats during your work.');
      }
    }
  },

  utils: {
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
        return;
      }

      // Type guard functions to narrow down what the element is
      const isInput = (el: Element): el is HTMLInputElement => {
        return el.tagName === 'INPUT';
      };
      const isTextarea = (el: Element): el is HTMLTextAreaElement => {
        return el.tagName === 'TEXTAREA';
      };

      // process different form elements (traverse)
      const values = {};
      Array.from(form.children).forEach(element => {
        if (isTextarea(element)) {
          values[element.name] = element.value;
        }
        if (isInput(element)) {
          const type = element.getAttribute('type');
          if (['text', 'hidden', 'password'].includes(type)) {
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

    arrayToPostData: function (data: Record<string, string|number>): string {
      /**
       * Generates a URL query string from an object.
       *
       * @param data: The object from which to make a string.
       */
      if (data === null) { return null; }
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
        OZONE.ajax._javascriptLoadLock &&
        (new Date()).getTime() < OZONE.ajax._javascriptLoadLock + 2000
      ) {
        setTimeout(
          () => OZONE.utils.addJavascriptUrl(url, onLoadCallback, noReload), 50
        );
        return;
      }

      // Unset the lock
      OZONE.ajax._javascriptLoadLock = false;

      // Check if this script is already present
      const head = document.getElementsByTagName('head').item(0);
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
      OZONE.ajax._javascriptLoadLock = (new Date()).getTime();
      const newScriptElement = document.createElement('script');
      newScriptElement.setAttribute('type', 'text/javascript');
      newScriptElement.setAttribute('src', url);

      // When the new element has finished loading, unset the lock and fire the
      // callback
      YAHOO.util.Event.addListener(newScriptElement, 'load', function () {
        OZONE.ajax._javascriptLoadLock = false;
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
      const head = document.getElementsByTagName('head').item(0);
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
        OZONE.utils.formatDates(elementId);
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

    formatDates: function (topElementId: string): void {
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
       * @param topElementId: The ID of element whose children should be
       * searched for span.odate
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

      const topElement = document.getElementById(topElementId);
      const dateElements = topElement.querySelectorAll('span.odate');

      (Array.from(dateElements) as HTMLElement[]).forEach(dateElement => {
        let dateString: string;
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

          // XXX This isn't valid, but it might be referred to somewhere:
          dateElement.timestamp = timestamp;

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
              zoneoffset = ((zoneoffset < 10) ? '0' + zoneoffset : zoneoffset) + '00';
              zone = (zoneoffset > 0) ? '+' + zoneoffset : '-' + zoneoffset;
            }
            dateString = dateString.replace(/%z/ig, zone);
          }
          if (dateString.match(/%O/) || options.match(/agohover/)) {
            // time ago
            const secAgo = OZONE.request.timestamp - timestamp;
            secAgo += Math.floor(((new Date()).getTime() - OZONE.request.date.getTime()) * 0.001);
            const agoString = OZONE.utils.calculateDateAgo(secAgo);

            dateString = dateString.replace(/%O/, agoString);
            if (options.match(/agohover/)) {
              const hovertext = agoString + ' ago';
              OZONE.dialog.hovertip.makeTip(dateElement, { text: hovertext, style: { width: 'auto' } });
              YAHOO.util.Event.addListener(dateElement, 'mouseover', function (_event) {
                let secAgo = OZONE.request.timestamp - this.timestamp;
                secAgo += Math.floor(((new Date()).getTime() - OZONE.request.date.getTime()) * 0.001);
                const agoString = OZONE.utils.calculateDateAgo(secAgo);
                this.hovertip.getElementsByTagName('div').item(0).innerHTML = agoString + ' ' + ogettext('ago');
              });
            }
          }
        }
        if (dateString) {
          dateElement.innerHTML = dateString;
          dateElement.style.visibility = 'visible';
          dateElement.style.display = 'inline';
        }
      })
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

    /**
     * This is tricky. Loads desired url with parameters but the parameters
     * are contained in the POST body.
     */
    loadPage: function (url, parameters) {
      /**
       * ?? (unused)
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
      document.getElementsByTagName('body').item(0).appendChild(form);
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

    /**
     * Filter all substrings of form e.g. [[olang en:sdadsd|pl:asdadad|]]
     */
    olang: function (text: string): string {
      /**
       * ??
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
      });
    }

  },

  lang: 'en', // default language

  loc: {
    // TODO What is loc? Localisation messages?
    messages: {},
    addMessages: function (mlist, lang) {
      /**
       * ??
       *
       * @param mlist: ??
       * @param lang: ??
       */
      if (!OZONE.loc.messages[lang]) {
        OZONE.loc.messages[lang] = {};
      }
      for (const i in mlist) {
        OZONE.loc.messages[lang][i] = mlist[i];
      }
    },
    addMessage: function (mid, mtr, lang) {
      /**
       * ??
       *
       * @param mid: ??
       * @param mtr: ??
       * @param lang: ??
       */
      if (!OZONE.loc.messages[lang]) {
        OZONE.loc.messages[lang] = {};
      }
      OZONE.loc.messages[lang][mid] = mtr;
    },
    getMessage: function (mid, lang) {
      /**
       * ??
       *
       * @param mid: ??
       * @param lang: ??
       */
      if (OZONE.loc.messages[lang]) {
        if (OZONE.loc.messages[lang][mid]) {
          return OZONE.loc.messages[lang][mid];
        }
      }
      // fall back to default
      return mid;
    }
  },

  visuals: {
    cursorWait: function () {
      const body = document.getElementsByTagName('body')[0];
      YAHOO.util.Dom.addClass(body, 'wait');
    },

    cursorClear: function () {
      const body = document.getElementsByTagName('body')[0];
      YAHOO.util.Dom.removeClass(body, 'wait');
    },

    scrollTo: function (elementId, options) {
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

  },

  forms: {
    lengthLimiter: function (
      textElementId: string,
      countElementId: string,
      limit: number
    ): void {
      /**
       * Constructs an object representing a length limiter for a text box.
       *
       * This function is supposed be called with `new`, but so far as I can
       * tell, the only module that actually uses the return value from that is
       * ManagerSiteLicenseModule.js. TODO Investigate further, possibly
       * refactor this into a class, possibly outsource this function
       *
       * @param textElementId: The ID of the element who content length should
       * be checked (input/textarea)
       * @param countElement: The ID of the element that contains text
       * representing the number of characters remaining.
       * @param limit: The character limit on the text box.
       */
      this.keyListener = () => {
        /**
         * Function called after every key to truncate the form text.
         */
        // get number of characters...
        let chars = this.textElement.value.replace(/\r\n/, '\n').length;
        this.countElement.innerHTML = this.limit - chars;
        if (chars > this.limit) {
          const scrollTop = this.textElement.scrollTop;
          this.textElement.value = this.textElement.value.substr(0, this.limit);
          this.textElement.scrollTop = scrollTop;
          chars = this.textElement.value.replace(/\r\n/, '\n').length;
          this.countElement.innerHTML = this.limit - chars;
        }
      };
      this.textElement = document.getElementById(textElementId);
      this.countElement = document.getElementById(countElementId);
      this.limit = limit;

      YAHOO.util.Event.addListener(this.textElement, 'keyup', this.keyListener, this, true);
      this.keyListener();
    }
  },

  dom: {
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
  },

  request: {},

  init: {} // This was an empty function
};

// OZONE.init();

export default OZONE;
