import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

export const AccountModeratorOfModule = {
  vars: {
    currentSiteId: null as null | number
  },
  listeners: {
    resign: function (_event: Event | null, siteId: number, siteName: string): void {
      AccountModeratorOfModule.vars.currentSiteId = siteId;
      const w = new OZONE.dialogs.ConfirmationDialog();
      w.content = document.getElementById("moderator-resign-dialog")!.innerHTML.replace(/%%SITE_NAME%%/, siteName);
      w.buttons = ['cancel', 'yes, resign'];
      w.addButtonListener('cancel', w.close);
      w.addButtonListener('yes, resign', AccountModeratorOfModule.listeners.resign2);
      w.show();
    },
    resign2: function (_event?: Event | null): void {
      const siteId = AccountModeratorOfModule.vars.currentSiteId;
      const params: RequestModuleParameters = {
        action: 'AccountMembershipAction',
        event: 'moderatorResign',
        site_id: siteId,
      };
      OZONE.ajax.requestModule(null, params, AccountModeratorOfModule.callbacks.resign);
    }
  },
  callbacks: {
    resign: function (response: YahooResponse): void {
      if (response.status == 'ok') {
        const w = new OZONE.dialogs.SuccessDialog();
        w.content = "You are no longer a moderator of this site.";
        w.show();
        Wikijump.modules.AccountModule.utils.loadModule('am-moderatorof');
      } else {
        const w = new OZONE.dialogs.ErrorDialog();
        w.content = response.message;
        w.show();
      }
    }
  }
};
