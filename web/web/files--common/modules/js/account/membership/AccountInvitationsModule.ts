import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

export const AccountInvitationsModule = {
  listeners: {
    acceptInvitation: function (_event: Event | null, invitationId: number): void {
      const params: RequestModuleParameters = {
        action: 'AccountMembershipAction',
        event: 'acceptInvitation',
        invitation_id: invitationId,
      };
      OZONE.ajax.requestModule(null, params, AccountInvitationsModule.callbacks.acceptInvitation);
    },
    throwAwayInvitation: function (_event: Event | null, invitationId: number): void {
      const params: RequestModuleParameters = {
        action: 'AccountMembershipAction',
        event: 'throwAwayInvitation',
        invitation_id: invitationId,
      };
      OZONE.ajax.requestModule(null, params, AccountInvitationsModule.callbacks.throwAwayInvitation);
    }
  },
  callbacks: {
    acceptInvitation: function (response: YahooResponse): void {
      if (response.status == 'ok') {
        const w = new OZONE.dialogs.SuccessDialog();
        w.content = response.message;
        w.show();
        Wikijump.modules.AccountModule.utils.loadModule("am-invitations");
      }
    },
    throwAwayInvitation: function (response: YahooResponse): void {
      if (response.status == 'ok') {
        Wikijump.modules.AccountModule.utils.loadModule("am-invitations");
      }
    }
  }
};
