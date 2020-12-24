import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

export const ManageSiteInvitationsHistoryModule = {
  vars: {
    showAll: null as null | boolean,
    invitationId: null as null | number,
  },
  listeners: {
    deleteInvitation: function (_event: Event | null, invitationId: number, email: string): void {
      if (confirm(`Are you sure you want to delete the invitation for ${email}?`)) {
        const params: RequestModuleParameters = {
          action: "ManageSiteMembershipAction",
          event: "deleteEmailInvitation",
          invitationId: invitationId
        };

        OZONE.ajax.requestModule(null, params, ManageSiteInvitationsHistoryModule.callbacks.deleteInvitation);
      }
    },
    resendInvitation: function (_event: Event | null, invitationId: number, rname: string, email: string): void {
      document.getElementById("resend-invitations-to")!.innerHTML = `${rname} &lt;${email}&gt;`;
      document.getElementById("resend-invitations-form")!.style.display = "block";

      OZONE.visuals.scrollTo("resend-invitations-form");

      ManageSiteInvitationsHistoryModule.vars.invitationId = invitationId;
    },
    resendInvitation2: function (_event?: Event | null): void {
      const invitationId = ManageSiteInvitationsHistoryModule.vars.invitationId;
      const params: RequestModuleParameters = {
        action: "ManageSiteMembershipAction",
        event: "resendEmailInvitation",
        message: (<HTMLTextAreaElement>document.getElementById("resend-invitations-message")!).value,
        invitationId: invitationId
      };
      OZONE.ajax.requestModule(null, params, ManageSiteInvitationsHistoryModule.callbacks.resendInvitation2);
    },
    showAdminOnly: function (_event?: Event | null): void {
      Wikijump.modules.ManageSiteModule.utils.loadModule('sm-invitations-history');
    },
    showAll: function (_event?: Event | null): void {
      Wikijump.modules.ManageSiteModule.utils.loadModule('sm-invitations-history', { showAll: true });
    }
  },
  callbacks: {
    deleteInvitation: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      // reload the module too
      Wikijump.modules.ManageSiteModule.utils.loadModule('sm-invitations-history', { showAll: ManageSiteInvitationsHistoryModule.vars.showAll });
    },
    resendInvitation2: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      // reload the module too
      Wikijump.modules.ManageSiteModule.utils.loadModule('sm-invitations-history', { showAll: ManageSiteInvitationsHistoryModule.vars.showAll });
    }
  },
  init: function (): void {
    // format dates
    OZONE.utils.formatDates("invitations-history-table");
    let showAll = true;
    if (document.getElementById("sm-invhist-showadminonly")!.style.fontWeight == "bold") {
      showAll = false;
    }
    ManageSiteInvitationsHistoryModule.vars.showAll = showAll;
  }
};

ManageSiteInvitationsHistoryModule.init();
