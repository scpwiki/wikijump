import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

export const ASInvitationsModule = {
  listeners: {
    save: function (_event?: Event | null): void {
      const val = (<HTMLInputElement>document.getElementById("receive-invitations-ch")!).checked;
      const params: RequestModuleParameters = {
        action: "AccountSettingsAction",
        event: "saveReceiveInvitations"
      };
      if (val) {
        params.receive = true;
      }
      OZONE.ajax.requestModule(null, params, ASInvitationsModule.callbacks.save);
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
