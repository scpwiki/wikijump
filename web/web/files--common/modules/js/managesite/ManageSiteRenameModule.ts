import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

declare const HTTP_SCHEMA: 'http' | 'https';
declare const URL_DOMAIN: string;

export const ManageSiteRenameModule = {
  vars: {
    currentCategory: null
  },
  listeners: {
    renameSite: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        unixName: (<HTMLInputElement>document.getElementById("sm-rename-site-unixname")!).value,
        action: 'ManageSiteAction',
        event: 'renameSite'
      };
      OZONE.ajax.requestModule(null, params, ManageSiteRenameModule.callbacks.renameSite);
      const w = new OZONE.dialogs.WaitBox();
      w.content = "Changing the URL...";
      w.show();
    }
  },
  callbacks: {
    renameSite: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const w = new OZONE.dialogs.SuccessBox();
      w.content = "The URL has been changed.";
      w.show();

      setTimeout(() => {
        window.location.href=`${HTTP_SCHEMA}://${response.unixName}.${URL_DOMAIN}`;
      }, 500);
    }

  }
};
