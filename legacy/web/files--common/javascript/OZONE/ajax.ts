import OZONE from ".";

declare const YAHOO: any;
declare type YahooResponse = any;
declare type YahooCallback = (response: any, arg?: unknown) => void;

// XXX Trying to type the module parameters is an exercise in futility - they
// will be attached to runData, from which parameters will be taken in
// AjaxModuleWikiFlowController, and remaining parameters will go directly to
// the module, where there is no parameter standardisation
export type RequestModuleParameters = {
  [key: string]: string | number | null | undefined;
};

type RequestModuleOptions = {
  clearRequestQueue: boolean;
};

export const ajax = {

  _callbackArray: [] as { callback: YahooCallback; arg: unknown }[],
  _callbackArrayIndex: 0,

  /**
   * arg - extra parameter passed to the callback as a second parameter
   */
  requestModule: function (
    moduleName: string | null,
    parameters: RequestModuleParameters,
    callback: YahooCallback,
    arg?: unknown,
    options?: RequestModuleOptions
  ): void {
    /**
     * ??
     *
     * @param moduleName: The name of the module being requested, or null if
     * some condition (??)
     * @param parameters: Parameters to pass to the callback.
     * Looks like they're attached to runData.
     * @param callback: ??
     * @param arg: Extra parameter passed to the callback
     * @param options: ??
     */
    OZONE.visuals.cursorWait();
    parameters ??= {}
    if (moduleName === null || moduleName === '') {
      moduleName = 'Empty';
    }
    parameters.moduleName = moduleName;

    if (options && options.clearRequestQueue) {
      ajax._callbackArray = [];
    }

    // TODO The callbackIndex can probably be refactored out
    const callbackIndex = ajax._callbackArrayIndex++;
    ajax._callbackArray[callbackIndex] = { callback, arg };

    parameters.callbackIndex = callbackIndex;

    YAHOO.util.Connect.asyncRequest(
      'POST',
      '/ajax--handler',
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
};
