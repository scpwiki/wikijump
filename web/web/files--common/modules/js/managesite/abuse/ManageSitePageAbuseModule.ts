import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

export const ManageSitePageAbuseModule = {
  listeners: {
    clear: function (_event: Event | null, path: string): void {
      const params: RequestModuleParameters = {
        action: "ManageSiteAbuseAction",
        event: "clearPageFlags",
        path
      };
      OZONE.ajax.requestModule(null, params, ManageSitePageAbuseModule.callbacks.clear);
    }
  },
  callbacks: {
    clear: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const w = new OZONE.dialogs.SuccessBox();
      w.content = "Flags cleared";
      w.show();

      Wikijump.modules.ManageSiteModule.utils.loadModule("sm-abuse-page");
    }
  }
};
