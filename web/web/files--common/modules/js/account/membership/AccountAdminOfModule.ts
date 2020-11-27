import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

export const AccountAdminOfModule = {
  vars: {
    currentSiteId: null as null | number
  },
  listeners: {
    resign: function (_event: Event | null, siteId: number, siteName: string): void {
      AccountAdminOfModule.vars.currentSiteId = siteId;
      const w = new OZONE.dialogs.ConfirmationDialog();
      w.content = document.getElementById("admin-resign-dialog")!.innerHTML.replace(/%%SITE_NAME%%/, siteName);
      w.buttons = ['cancel', 'yes, resign'];
      w.addButtonListener('cancel', w.close);
      w.addButtonListener('yes, resign', AccountAdminOfModule.listeners.resign2);
      w.show();
    },
    resign2: function (_event?: Event | null): void {
      const siteId = AccountAdminOfModule.vars.currentSiteId;
      const params: RequestModuleParameters = {
        action: 'AccountMembershipAction',
        event: 'adminResign',
        site_id: siteId
      };
      OZONE.ajax.requestModule(null, params, AccountAdminOfModule.callbacks.resign);
    }
  },
  callbacks: {
    resign: function (response: YahooResponse): void {
      if (response.status == 'ok') {
        const w = new OZONE.dialogs.SuccessDialog();
        w.content = "You are no longer an admin of this site.";
        w.show();
        Wikijump.modules.AccountModule.utils.loadModule('am-adminof');
      } else {
        const w = new OZONE.dialogs.ErrorDialog();
        w.content = response.message;
        w.show();
      }
    }
  }
};
