import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

export const AccountApplicationsModule = {
  vars: {
    currentSiteId: null as null | number,
  },
  listeners: {
    remove: function (_event: Event | null, siteId: number, siteName: string): void {
      AccountApplicationsModule.vars.currentSiteId = siteId;
      const w = new OZONE.dialogs.ConfirmationDialog();
      w.content = document.getElementById("application-remove-dialog")!.innerHTML.replace(/%%SITE_NAME%%/, siteName);
      w.buttons = ['cancel', 'yes, remove'];
      w.addButtonListener('cancel', w.close);
      w.addButtonListener('yes, remove', AccountApplicationsModule.listeners.remove2);
      w.show();
    },
    remove2: function (_event?: Event | null, siteId0?: number, _siteName0?: string): void {
      const siteId = siteId0 ?? AccountApplicationsModule.vars.currentSiteId;
      const params: RequestModuleParameters = {
        action: 'AccountMembershipAction',
        event: 'removeApplication',
        site_id: siteId
      };
      OZONE.ajax.requestModule(null, params, AccountApplicationsModule.callbacks.remove);
    }
  },
  callbacks: {
    remove: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const w = new OZONE.dialogs.SuccessBox();
      w.content = "The application has been removed.";
      w.show();
      Wikijump.modules.AccountModule.utils.loadModule('am-applications');
    }
  }
};
