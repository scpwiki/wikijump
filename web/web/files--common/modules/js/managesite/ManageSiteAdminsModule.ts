import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

export const ManageSiteAdminsModule = {
  vars: {
    currentUserId: null as null | number
  },
  listeners: {
    removeAdmin: function (userId: number, userName: string): void {
      ManageSiteAdminsModule.vars.currentUserId = userId;
      const w = new OZONE.dialogs.ConfirmationDialog();
      w.content = document.getElementById("remove-admin-dialog")!.innerHTML.replace(/%%USER_NAME%%/, userName);
      w.buttons = ['cancel', 'yes, remove'];
      w.addButtonListener('cancel', w.close);
      w.addButtonListener('yes, remove', ManageSiteAdminsModule.listeners.removeAdmin2);
      w.show();
    },
    removeAdmin2: function (_event?: Event | null): void {
      const userId = ManageSiteAdminsModule.vars.currentUserId;
      const params: RequestModuleParameters = {
        action: 'ManageSiteMembershipAction',
        event: 'removeAdmin',
        user_id: userId
      };
      OZONE.ajax.requestModule(null, params, ManageSiteAdminsModule.callbacks.removeAdmin);
    }
  },
  callbacks: {
    removeAdmin: function (response: YahooResponse): void {
      if (response.status == 'ok') {
        const w = new OZONE.dialogs.SuccessDialog();
        w.content = "The user has been removed from site administrators.";
        w.show();
      } else {
        const w = new OZONE.dialogs.ErrorDialog();
        w.content = response.message;
        w.show();
      }
      Wikijump.modules.ManageSiteModule.utils.loadModule('sm-admins');
    }
  }
};
