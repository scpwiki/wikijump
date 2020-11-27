import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare const YAHOO: any;
declare type YahooResponse = any;

export const APAboutModule = {
  listeners: {
    aboutChange: function (_event?: Event | null): void {
      // get number of characters...
      const aboutTextarea = <HTMLTextAreaElement>document.getElementById("about-textarea")!;
      const chars = aboutTextarea.value.replace(/\response\n/, "\n").length;
      document.getElementById("chleft")!.innerHTML = `${200 - chars}`;
      if (chars > 200) {
        const scrollTop = aboutTextarea.scrollTop;
        aboutTextarea.value = aboutTextarea.value.substr(0, 200);
        aboutTextarea.scrollTop = scrollTop;
        const chars = aboutTextarea.value.replace(/\response\n/, "\n").length;
        document.getElementById("chleft")!.innerHTML = `${200 - chars}`;
      }
    },
    save: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        ...OZONE.utils.formToArray("about-form"),
        action: "AccountProfileAction",
        event: "saveAbout",
      };
      OZONE.ajax.requestModule(null, params, APAboutModule.callbacks.save);
      const w = new OZONE.dialogs.WaitBox();
      w.content = "Saving profile information...";
      w.show();
    }
  },
  callbacks: {
    save: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }

      const w = new OZONE.dialogs.SuccessBox();
      w.content = "Your profile information has been saved.";
      w.show();
    }
  },
  init: function (): void {
    YAHOO.util.Event.addListener("about-textarea", "keyup", APAboutModule.listeners.aboutChange);
    APAboutModule.listeners.aboutChange();
  }
};

  APAboutModule.init();
