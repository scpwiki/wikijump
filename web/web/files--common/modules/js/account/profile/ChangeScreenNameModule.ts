import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare const YAHOO: any;
declare type YahooResponse = any;

export const ChangeScreenNameModule = {
  listeners: {
    save: function (event: Event): void {
      const params: RequestModuleParameters = {
        action: "AccountProfileAction",
        event: "changeScreenName",
        screenName: (<HTMLInputElement>document.getElementById("ap-screen-name-input")!).value
      };
      OZONE.ajax.requestModule(null, params, ChangeScreenNameModule.callbacks.save);
      const w = new OZONE.dialogs.WaitBox();
      w.content = "Changing the screen name...";
      w.show();
      YAHOO.util.Event.stopEvent(event);
    }
  },
  callbacks: {
    save: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }

      const w = new OZONE.dialogs.SuccessBox();
      w.content = "Your screen name has been changed!";
      w.show();

      OZONE.ajax.requestModule('account/profile/ChangeScreenNameModule', {}, Wikijump.modules.AccountModule.callbacks.menuClick);
    }
  }
};
