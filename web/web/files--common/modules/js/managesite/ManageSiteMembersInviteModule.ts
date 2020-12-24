import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare const YAHOO: any;
declare type YahooResponse = any;

export const ManageSiteMembersInviteModule = {
  vars: {
    userId: null as null | string,
    userNames: {} as { [id: string]: string },
    searchCount: 0
  },
  listeners: {
    searchClick: function (event: Event): void {
      const query = (<HTMLInputElement>document.getElementById("sm-mi-search-f")!).value;
      YAHOO.util.Event.preventDefault(event);
      if (query.length < 2) {
        document.getElementById("sm-mi-search-response")!.innerHTML = "please type at least 2 characters...";
        return;
      }
      const params: RequestModuleParameters = {
        query: query
      };
      OZONE.ajax.requestModule("users/UserSearchModule", params, ManageSiteMembersInviteModule.callbacks.searchClick);
    },
    inviteMember: function (_event?: Event | null): void {
      // display a nice dialog box...
      // @ts-expect-error What is this?
      const userId = this.id.replace("invite-member-b-", '');
      const userName = ManageSiteMembersInviteModule.vars.userNames[userId];
      ManageSiteMembersInviteModule.vars.userId = userId;

      const w = new OZONE.dialogs.Dialog();
      const reg = new RegExp('template-id-stub', 'g');
      w.content = document.getElementById("sm-tmp-not")!.innerHTML.replace(reg, 's').replace(/%%USERNAME%%/g, userName);
      w.show();

      YAHOO.util.Event.addListener("s-cancel", "click", ManageSiteMembersInviteModule.listeners.cancelInvitation);
      YAHOO.util.Event.addListener("s-send", "click", ManageSiteMembersInviteModule.listeners.inviteMember2);
      new OZONE.forms.lengthLimiter("s-text", "s-charleft", 200);
    },
    cancelInvitation: function (): void {
      const container = OZONE.dialog.factory.boxcontainer();
      container.hideContent();
      OZONE.dialog.cleanAll();
    },
    inviteMember2: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        action: 'ManageSiteMembershipAction',
        event: "inviteMember",
        user_id: ManageSiteMembersInviteModule.vars.userId,
        text: (<HTMLInputElement>document.getElementById("s-text")!).value
      };
      OZONE.ajax.requestModule("Empty", params, ManageSiteMembersInviteModule.callbacks.inviteMember);
    }
  },
  callbacks: {
    searchClick: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }

      OZONE.utils.setInnerHTMLContent("sm-mi-search-response", response.body);
      ManageSiteMembersInviteModule.vars.searchCount = response.count;

      // now modify the fields to include "add as member"

      const userIds = response.userIds;
      for (let i = 0; i < userIds.length; i++) {
        const	divid = "found-user-" + userIds[i];
        const el = document.getElementById(divid)!;
        el.innerHTML += '(<a href="javascript:;" id="invite-member-b-' + userIds[i] + '">invite</a>)';
        YAHOO.util.Event.addListener("invite-member-b-" + userIds[i], "click", ManageSiteMembersInviteModule.listeners.inviteMember);
      }

      ManageSiteMembersInviteModule.vars.userNames = response.userNames;
    },
    inviteMember: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }

      const w = new OZONE.dialogs.SuccessBox();
      w.content = "The invitation has been sent";
      w.show();
    }
  },
  init: function (): void {
    YAHOO.util.Event.addListener("sm-mi-search-b", "click", ManageSiteMembersInviteModule.listeners.searchClick);
    YAHOO.util.Event.addListener("sm-search-user", "submit", ManageSiteMembersInviteModule.listeners.searchClick);
  }
};

ManageSiteMembersInviteModule.init();
