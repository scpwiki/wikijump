import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

// Defined in Wikilayout.tpl
declare const HTTP_SCHEMA: string;
declare const URL_HOST: string;

export const ASLanguageModule = {
  listeners: {
    save: function (_event?: Event | null): void {
      const lang = (<HTMLSelectElement>document.getElementById("as-language-select")!).value;
      const params: RequestModuleParameters = {
        action: "AccountSettingsAction",
        event: "saveLanguage",
        language: lang
      };
      const w = new OZONE.dialogs.WaitBox();
      w.content = "Saving changes...";
      w.show();

      OZONE.ajax.requestModule(null, params, ASLanguageModule.callbacks.save);
    }
  },
  callbacks: {
    save: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }

      const w = new OZONE.dialogs.SuccessBox();
      w.content = "Changes saved";
      w.show();
      const url = `${HTTP_SCHEMA}://${URL_HOST}/account:you`;
      setTimeout(() => window.location.href = url, 1500);
    }

  }
};
