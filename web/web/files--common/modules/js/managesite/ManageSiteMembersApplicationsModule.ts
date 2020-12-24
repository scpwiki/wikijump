import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

export const ManageSiteMembersApplicationsModule = {
  vars: {
    currentUserId: null as null | number,
    type: null as null | 'accept' | 'decline'
  },
  listeners: {
    accept: function (_event: Event | null, userId: number, userName: string, type: 'accept' | 'decline'): void {
      ManageSiteMembersApplicationsModule.vars.currentUserId = userId;
      ManageSiteMembersApplicationsModule.vars.type = type;
      const w = new OZONE.dialogs.Dialog();
      w.title = "Membership application - " + type;
      w.content = document.getElementById("dialog43")!.innerHTML.replace(/template-id-stub-/g, 'a-').replace(/%%TYPE%%/g, type).replace(/%%USER_NAME%%/, userName);
      w.buttons = ['cancel', 'send decision'];
      w.addButtonListener('cancel', w.close);
      w.addButtonListener('send decision', ManageSiteMembersApplicationsModule.listeners.accept2);
      w.show();
      // Initially template-id-stub-app-area
      new OZONE.forms.lengthLimiter("a-app-area", "a-app-area-left", 200);
    },
    accept2: function (_event?: Event | null): void {
      const userId = ManageSiteMembersApplicationsModule.vars.currentUserId;
      const params: RequestModuleParameters = {
        action: 'ManageSiteMembershipAction',
        event: 'acceptApplication',
        user_id: userId,
        // Initially template-id-stub-app-area
        text: (<HTMLTextAreaElement>document.getElementById("a-app-area")!).value,
        type: ManageSiteMembersApplicationsModule.vars.type
      };

      OZONE.ajax.requestModule(null, params, ManageSiteMembersApplicationsModule.callbacks.accept);
    }
  },
  callbacks: {
    accept: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }

      const w = new OZONE.dialogs.SuccessBox();
      w.content = "The decision has been sent.";
      w.show();
      Wikijump.modules.ManageSiteModule.utils.loadModule('sm-ma');
    }
  }
};
