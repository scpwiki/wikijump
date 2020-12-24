import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

export const ManageSiteMembersListModule = {
  vars: {
    currentUserId: null as null | number
  },
  removeUser: function (userId: number, userName: string): void {
    ManageSiteMembersListModule.vars.currentUserId = userId;
    const w = new OZONE.dialogs.ConfirmationDialog();
    w.content = document.getElementById("remove-user-dialog")!.innerHTML.replace(/%%USER_NAME%%/, userName);
    w.buttons = ['cancel', 'yes, remove'];
    w.addButtonListener('cancel', w.close);
    w.addButtonListener('yes, remove', ManageSiteMembersListModule.listeners.removeUser2);
    w.show();
  },
  toModerators: function (userId: number): void {
    const params: RequestModuleParameters = {
      action: 'ManageSiteMembershipAction',
      event: 'toModerators',
      user_id: userId
    };
    OZONE.ajax.requestModule(null, params, ManageSiteMembersListModule.callbacks.toModerators);
  },
  toAdmins: function (userId: number): void {
    const params: RequestModuleParameters = {
      action: 'ManageSiteMembershipAction',
      event: 'toAdmins',
      user_id: userId
    };
    OZONE.ajax.requestModule(null, params, ManageSiteMembersListModule.callbacks.toAdmins);
  },
  listeners: {
    removeUser2: function (_event?: Event | null): void {
      const userId = ManageSiteMembersListModule.vars.currentUserId;
      const params: RequestModuleParameters = {
        action: 'ManageSiteMembershipAction',
        event: 'removeMember',
        user_id: userId
      };
      OZONE.ajax.requestModule(null, params, ManageSiteMembersListModule.callbacks.removeUser);
    },
    removeAndBan: function (userId: number, userName: string): void {
      ManageSiteMembersListModule.vars.currentUserId = userId;
      const w = new OZONE.dialogs.ConfirmationDialog();
      w.content = document.getElementById("remove-ban-user-dialog")!.innerHTML.replace(/%%USER_NAME%%/, userName);
      w.buttons = ['cancel', 'yes, remove and ban'];
      w.addButtonListener('cancel', w.close);
      w.addButtonListener('yes, remove and ban', ManageSiteMembersListModule.listeners.removeAndBan2);
      w.show();
    },
    removeAndBan2: function (_event?: Event | null): void {
      const userId = ManageSiteMembersListModule.vars.currentUserId;
      const params: RequestModuleParameters = {
        action: 'ManageSiteMembershipAction',
        event: 'removeMember',
        ban: 'yes',
        user_id: userId
      };
      OZONE.ajax.requestModule(null, params, ManageSiteMembersListModule.callbacks.removeAndBan);
    }
  },
  callbacks: {
    removeUser: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }

      const w = new OZONE.dialogs.SuccessDialog();
      w.content = "The user has been removed.";
      w.show();

      Wikijump.modules.ManageSiteModule.utils.loadModule('sm-members-list');
    },
    toModerators: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }

      const w = new OZONE.dialogs.SuccessDialog();
      w.content = `The user <strong>${response.userName}</strong> has been added to moderators.<br/>Now please go to the list of moderators and set new permissions.`;
      w.show();
    },
    toAdmins: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }

      const w = new OZONE.dialogs.SuccessDialog();
      w.content = "The user <strong>" + response.userName + "</strong> has been added to site administrators.";
      w.show();
    },
    removeAndBan: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }

      const w = new OZONE.dialogs.SuccessDialog();
      w.content = "The user has been removed and banned.";
      w.show();

      Wikijump.modules.ManageSiteModule.utils.loadModule('sm-members-list');
    }
  }
};
