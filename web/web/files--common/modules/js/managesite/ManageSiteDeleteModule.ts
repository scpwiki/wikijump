import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

declare const HTTP_SCHEMA: string;
declare const URL_DOMAIN: string;

export const ManageSiteDeleteModule = {
  vars: {
    currentCategory: null
  },
  listeners: {
    deleteSite: function (_event?: Event | null): void {
      OZONE.ajax.requestModule("managesite/ManageSiteDelete2Module", {}, ManageSiteDeleteModule.callbacks.deleteSite);
    },
    deleteSite2: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        action: "ManageSiteAction",
        event: "DeleteSite"
      };
      OZONE.ajax.requestModule(null, params, ManageSiteDeleteModule.callbacks.deleteSite2);
      const w = new OZONE.dialogs.WaitBox();
      w.content = "Deleting the site...";
      w.show();
    }
  },
  callbacks: {
    deleteSite: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      document.getElementById("sm-delete-box")!.innerHTML = response.body;
    },
    deleteSite2: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const w = new OZONE.dialogs.SuccessBox();
      w.content = "The site has been deleted.";
      setTimeout(() => window.location.href=`${HTTP_SCHEMA}://${URL_DOMAIN}/account:you/start/deletedsites`, 1000);
      w.show();
    }
  }
};
