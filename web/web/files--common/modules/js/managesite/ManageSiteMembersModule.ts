import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare const YAHOO: any;
declare type YahooResponse = any;

export const ManageSiteMembersModule = {
  listeners: {
    save: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        ...OZONE.utils.formToArray("sm-mem-form"),
        action: "ManageSiteMembershipAction",
        event: "saveMemberPolicy",
      };
      OZONE.ajax.requestModule("Empty", params, ManageSiteMembersModule.callbacks.save);
      const w = new OZONE.dialogs.WaitBox();
      w.content = "Saving changes...";
      w.show();
    },
    cancel: function (_event?: Event | null): void {
      OZONE.ajax.requestModule("managesite/ManageSiteModule", {}, ManageSiteMembersModule.callbacks.cancel);
    }
  },
  callbacks: {
    save: function (): void {
      const w = new OZONE.dialogs.SuccessBox();
      w.content = "Changes saved";
      w.show();
    },
    cancel: function (response: YahooResponse): void {
      OZONE.utils.setInnerHTMLContent("site-manager", response.body);
    }
  },
  init: function (): void {
    YAHOO.util.Event.addListener("sm-members-cancel", "click", ManageSiteMembersModule.listeners.cancel);
    YAHOO.util.Event.addListener("sm-members-save", "click", ManageSiteMembersModule.listeners.save);
  }
};

ManageSiteMembersModule.init();
