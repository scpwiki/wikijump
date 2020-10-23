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
//
// XXX Temporary until we deprecate Yahoo UI
type YahooResponse = {
  status: number
  statusText: string
  responseText: string
}

export const ajax = {

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
};
