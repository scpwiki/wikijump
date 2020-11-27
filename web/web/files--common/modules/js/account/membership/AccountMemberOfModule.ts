import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

export const AccountMemberOfModule = {
  vars: {
    signOffId: null as null | number
  },
  listeners: {
    signOff: function (_event: Event | null, [siteId, siteName]: [number, string]): void {
      AccountMemberOfModule.vars.signOffId = siteId;
      const w = new OZONE.dialogs.ConfirmationDialog();
      w.content = document.getElementById("signoff-window")!.innerHTML.replace(/%%SITE_NAME%%/, siteName);
      w.buttons = ['cancel', 'yes, sign me off'];
      w.addButtonListener('cancel', w.close);
      w.addButtonListener('yes, sign me off', AccountMemberOfModule.listeners.signOff2);
      w.show();
    },
    signOff2: function (_event?: Event | null): void {
      const siteId = AccountMemberOfModule.vars.signOffId;
      const params: RequestModuleParameters = {};
      params.site_id = siteId;
      params.action = 'AccountMembershipAction';
      params.event = 'signOff';
      OZONE.ajax.requestModule(null, params, AccountMemberOfModule.callbacks.signOff);
    }
  },
  callbacks: {
    signOff: function (response: YahooResponse): void {
      if (response.status == 'ok') {
        const w = new OZONE.dialogs.SuccessDialog();
        w.content = "You have successfully signed off from this site";
        w.show();
        Wikijump.modules.AccountModule.utils.loadModule("am-memberof");
      } else {
        const w = new OZONE.dialogs.ErrorDialog();
        w.content = response.message;
        w.show();
      }
    }
  }
};
