import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

export const ASNotificationsModule = {
  listeners: {
    saveReceiveDigest: function (_event?: Event | null): void {
      const receive = (<HTMLInputElement>document.getElementById("as-receive-digest")!).checked;
      const params: RequestModuleParameters = {
        action: "AccountSettingsAction",
        event: "saveReceiveDigest"
      };
      if (receive) {
        params.receive = "yes";
      }

      const w = new OZONE.dialogs.WaitBox();
      w.content = "Saving changes...";
      w.show();

      OZONE.ajax.requestModule(null, params, ASNotificationsModule.callbacks.saveReceiveDigest);
    },
    saveReceiveNewsletter: function (_event?: Event | null): void {
      const receive = (<HTMLInputElement>document.getElementById("as-receive-newsletter")!).checked;
      const params: RequestModuleParameters = {
        action: "AccountSettingsAction",
        event: "saveReceiveNewsletter"
      };
      if (receive) {
        params.receive = "yes";
      }

      const w = new OZONE.dialogs.WaitBox();
      w.content = "Saving changes...";
      w.show();

      OZONE.ajax.requestModule(null, params, ASNotificationsModule.callbacks.saveReceiveDigest);
    }
  },
  callbacks: {
    saveReceiveDigest: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }

      const w = new OZONE.dialogs.SuccessBox();
      w.content = "Changes saved";
      w.show();
    }

  }
};
