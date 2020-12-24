import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

export const ManageSiteSecureAccessModule = {
  listeners: {
    save: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        action: "ManageSiteAction",
        event: "saveSecureAccess",
        secureMode: (<HTMLSelectElement>document.getElementById("sm-ssl-mode-select")!).value
      };
      OZONE.ajax.requestModule(null, params, ManageSiteSecureAccessModule.callbacks.save);

      const w = new OZONE.dialogs.WaitBox();
      w.content = "Saving changes...";
      w.show();
    }
  },
  callbacks: {
    save: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const w = new OZONE.dialogs.SuccessBox();
      w.content = "Changes saved";
      w.show();

      // reload the page!
      window.location.reload();
    }
  }
};
