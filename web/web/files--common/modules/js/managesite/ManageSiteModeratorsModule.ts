import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

export const ManageSiteModeratorsModule = {
  vars: {
    currentUserId: null as null | number
  },
  listeners: {
    removeModerator: function (_event: Event | null, userId: number, userName: string): void {
      ManageSiteModeratorsModule.vars.currentUserId = userId;
      const w = new OZONE.dialogs.ConfirmationDialog();
      w.content = document.getElementById("remove-moderator-dialog")!.innerHTML.replace(/%%USER_NAME%%/, userName);
      w.buttons = ['cancel', 'yes, remove'];
      w.addButtonListener('cancel', w.close);
      w.addButtonListener('yes, remove', ManageSiteModeratorsModule.listeners.removeModerator2);
      w.show();
    },
    removeModerator2: function (_event?: Event | null): void {
      const userId = ManageSiteModeratorsModule.vars.currentUserId;
      const params: RequestModuleParameters = {
        action: 'ManageSiteMembershipAction',
        event: 'removeModerator',
        user_id: userId
      };
      OZONE.ajax.requestModule(null, params, ManageSiteModeratorsModule.callbacks.removeModerator);
    },
    moderatorPermissions: function (_event: Event | null, moderatorId: number): void {
      const params: RequestModuleParameters = {
        moderatorId: moderatorId
      };
      OZONE.ajax.requestModule("managesite/ManageSiteModeratorPermissionsModule", params, ManageSiteModeratorsModule.callbacks.moderatorPermissions);
    },
    cancelPermissions: function (_event: Event | null, moderatorId: number): void {
      const el = document.getElementById("mod-permissions-" + moderatorId)!;
      el.style.display = "none";
      el.innerHTML = '';
    },
    savePermissions: function (_event: Event | null): void {
      const params: RequestModuleParameters = {
        ...OZONE.utils.formToArray("sm-mod-perms-form"),
        action: 'ManageSiteMembershipAction',
        event: 'saveModeratorPermissions'
      };
      OZONE.ajax.requestModule(null, params, ManageSiteModeratorsModule.callbacks.savePermissions);
      const w = new OZONE.dialogs.WaitBox();
      w.content = "Saving permissions...";
      w.show();
    }
  },
  callbacks: {
    removeModerator: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const w = new OZONE.dialogs.SuccessDialog();
      w.content = "The user has been removed from site moderators.";
      w.show();
      Wikijump.modules.ManageSiteModule.utils.loadModule('sm-moderators');
    },
    moderatorPermissions: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      document.getElementById("mod-permissions-" + response.moderatorId)!.innerHTML = response.body;
      document.getElementById("mod-permissions-" + response.moderatorId)!.style.display = "block";
    },
    savePermissions: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const w = new OZONE.dialogs.SuccessBox();
      w.content = "Moderator permissions saved.";
      w.show();
    }
  }
};
