import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

export const ManageSiteUserAbuseModule = {
  vars: {
    currentUserId: null as null | number
  },
  listeners: {
    clear: function (_event: Event | null, userId: number): void {
      const params: RequestModuleParameters = {
        action: "ManageSiteAbuseAction",
        event: "clearUserFlags",
        userId
      };
      OZONE.ajax.requestModule(null, params, ManageSiteUserAbuseModule.callbacks.clear);
    },
    removeUser: function (userId: number, userName: string): void {
      ManageSiteUserAbuseModule.vars.currentUserId = userId;
      const w = new OZONE.dialogs.ConfirmationDialog();
      w.content = document.getElementById("remove-user-dialog")!.innerHTML.replace(/%%USER_NAME%%/, userName);
      w.buttons = ['cancel', 'yes, remove'];
      w.addButtonListener('cancel', w.close);
      w.addButtonListener('yes, remove', ManageSiteUserAbuseModule.listeners.removeUser2);
      w.show();
    },
    removeUser2: function (_event?: Event | null): void {
      const userId = ManageSiteUserAbuseModule.vars.currentUserId;
      const params: RequestModuleParameters = {
        action: 'ManageSiteMembershipAction',
        event: 'removeMember',
        user_id: userId
      };
      OZONE.ajax.requestModule(null, params, ManageSiteUserAbuseModule.callbacks.removeUser);
    },
    removeAndBan: function (userId: number, userName: string): void {
      ManageSiteUserAbuseModule.vars.currentUserId = userId;
      const w = new OZONE.dialogs.ConfirmationDialog();
      w.content = document.getElementById("remove-ban-user-dialog")!.innerHTML.replace(/%%USER_NAME%%/, userName);
      w.buttons = ['cancel', 'yes, remove and ban'];
      w.addButtonListener('cancel', w.close);
      w.addButtonListener('yes, remove and ban', ManageSiteUserAbuseModule.listeners.removeAndBan2);
      w.show();
    },
    removeAndBan2: function (_event?: Event | null): void {
      const userId = ManageSiteUserAbuseModule.vars.currentUserId;
      const params: RequestModuleParameters = {
        action: 'ManageSiteMembershipAction',
        event: 'removeMember',
        ban: 'yes',
        user_id: userId
      };
      OZONE.ajax.requestModule(null, params, ManageSiteUserAbuseModule.callbacks.removeAndBan);
    },
    banUser: function (userId: number, userName: string): void {
      ManageSiteUserAbuseModule.vars.currentUserId = userId;
      const w = new OZONE.dialogs.ConfirmationDialog();
      w.content = document.getElementById("ban-user-dialog")!.innerHTML.replace(/%%USER_NAME%%/, userName);
      w.buttons = ['cancel', 'yes, ban'];
      w.addButtonListener('cancel', w.close);
      w.addButtonListener('yes, ban', ManageSiteUserAbuseModule.listeners.banUser2);
      w.show();
    },
    banUser2: function (_event?: Event | null): void {
      const userId = ManageSiteUserAbuseModule.vars.currentUserId;
      const params: RequestModuleParameters = {
        userId: userId,
        action: "ManageSiteBlockAction",
        event: "blockUser"
      };
      OZONE.ajax.requestModule(null, params, ManageSiteUserAbuseModule.callbacks.banUser);
    }
  },
  callbacks: {
    clear: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const w = new OZONE.dialogs.SuccessBox();
      w.content = "Flags cleared";
      w.show();
    },
    removeUser: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }

      const w = new OZONE.dialogs.SuccessDialog();
      w.content = "The user has been removed.";
      w.show();

      Wikijump.modules.ManageSiteModule.utils.loadModule('sm-abuse-user');
    },
    removeAndBan: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }

      const w = new OZONE.dialogs.SuccessDialog();
      w.content = "The user has been removed and banned.";
      w.show();
      Wikijump.modules.ManageSiteModule.utils.loadModule('sm-abuse-user');
    },
    banUser: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }

      const w = new OZONE.dialogs.SuccessBox();
      w.content = "The user has been blocked.";
      w.show();
      Wikijump.modules.ManageSiteModule.utils.loadModule('sm-abuse-user');
    }
  }
};
