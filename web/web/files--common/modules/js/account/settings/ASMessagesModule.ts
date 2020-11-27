import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

export const ASMessagesModule = {
  listeners: {
    save: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        ...OZONE.utils.formToArray("receive-pl-form"),
        action: "AccountSettingsAction",
        event: "saveReceiveMessages"
      };
      OZONE.ajax.requestModule(null, params, ASMessagesModule.callbacks.save);
      const w = new OZONE.dialogs.WaitBox();
      w.content = "Saving preferences...";
      w.show();
    }
  },
  callbacks: {
    save: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const w = new OZONE.dialogs.SuccessBox();
      w.content = "Preferences saved.";
      w.show();
    }

  }
};
