import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

export const ManageSiteLetUsersInviteModule = {
  vars: {
  },
  listeners: {
    save: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        action: "ManageSiteMembershipAction",
        event: "letUsersInviteSave",
        enableLetUsersInvite: (<HTMLInputElement>document.getElementById("sm-allow-users-invite")!).checked
      };

      OZONE.ajax.requestModule(null, params, ManageSiteLetUsersInviteModule.callbacks.save);

      const w = new OZONE.dialogs.WaitBox();
      w.content = "Saving changes...";
      w.show();
    }
  },
  callbacks: {
    save: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const w = new OZONE.dialogs.SuccessBox();
      w.content = "Changes saved";
      w.show();
    }
  },
};
