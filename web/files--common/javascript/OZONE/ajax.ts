import OZONE from ".";
import YAHOO from "@/javascript/yahooui/types";

type RequestModuleParameters = {
  // TODO Add more values as we discover them
  action?: 'FileAction'
  event?: 'checkFileExists'
  moduleName?: string
  pageId?: number
  callbackIndex?: number
  wikijump_token7?: string /* eslint-disable-line camelcase */
}

type RequestModuleOptions = {
  clearRequestQueue: boolean
}

// XXX Temporary until we deprecate Yahoo UI
type YahooResponse = {
  // TODO Work out what else should be here
  status: number
  statusText: string
  responseText: string
}
type YahooCallback = (response: YahooResponse, arg?: unknown) => void

export const ajax = {

  _callbackArray: [] as { callback: YahooCallback, arg: unknown }[],
  _callbackArrayIndex: 0,

  /**
   * arg - extra parameter passed to the callback as a second parameter
   */
  requestModule: function (
    moduleName: string,
    parameters: RequestModuleParameters,
    callback: YahooCallback,
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
      ajax._callbackArray = [];
    }

    // TODO The callbackIndex can probably be refactored out
    const callbackIndex = ajax._callbackArrayIndex++;
    ajax._callbackArray[callbackIndex] = {
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
      ajax.requestModuleCallback,
      OZONE.utils.arrayToPostData(parameters)
    );
  },

  requestModuleCallback: {
    success: function (responseObject: YahooResponse): void {
      // Response comes from yahooui/connection.js/createResponseObject
      // TODO What is the type of the returned JSON?
      // XXX This is an implicit any - why no error?
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
      if (!ajax._callbackArray[callbackIndex]) {
        return;
      }
      const callback = ajax._callbackArray[callbackIndex].callback;
      if (!callback) {
        alert('internal: callback error');
      }
      const arg = ajax._callbackArray[callbackIndex].arg;
      // call callback

      if (arg !== null) {
        callback(response, arg);
      } else {
        callback(response);
      }

      // attach javascript (if any)
      // XXX Where do jsInclude and cssInclude come from? Where are they set?
      if (response.jsInclude != null) {
        response.jsInclude.forEach((jsUrl: string) => {
          OZONE.utils.addJavascriptUrl(jsUrl);
        });
      }
      if (response.cssInclude != null) {
        response.cssInclude.forEach((cssUrl: string) => {
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

  }

  // XXX While the concept of a quickmodule does appear in the codebase, it is
  // confined to the PHP, and the following two methods are unused. They are
  // not type-safe (and, because they are unused, not worth refactoring) so I
  // have commented them out.
  // requestQuickModule: function (
  //   moduleName: unknown,
  //   parameters: unknown,
  //   callback: (response: YahooResponse) => void
  // ): void {
  //   /**
  //    * ?? (unused)
  //    *
  //    * @param moduleName: ??
  //    * @param parameters: ??
  //    * @param callback: ??
  //    */
  //   if (parameters === null) {
  //     parameters = {};
  //   }
  //   if (moduleName === null || moduleName === '') {
  //     alert('Quick module name empty.');
  //   }

  //   const callbackIndex = ajax._callbackArrayIndex++;
  //   ajax._callbackArray[callbackIndex] = callback;

  //   parameters.callbackIndex = callbackIndex;

  //   const postdata = JSON.stringify(parameters);
  //   const internalCallback = ajax.requestQuickModuleCallback;
  //   YAHOO.util.Connect.asyncRequest('POST', '/quickmodule.php?module=' + moduleName, internalCallback, postdata);
  // },

  // requestQuickModuleCallback: {
  //   /**
  //    * ?? (unused)
  //    */
  //   success: function (responseObject: YahooResponse): void {
  //     // process response
  //     const response = JSON.parse(responseObject.responseText);
  //     const callbackIndex = response.callbackIndex;
  //     const callback = ajax._callbackArray[callbackIndex];
  //     callback(response);
  //   },
  //   failure: function (_responseObject: YahooResponse): void {
  //     alert('The ajax request failed. Please check your internet connection or\nreport a bug if the error repeats during your work.');
  //   }
  // }
};
