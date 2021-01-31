import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

export const AcceptTOSModule = {
  listeners: {
    nextClick: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        ...OZONE.utils.formToArray('accept-tos-form'),
        action: "CreateAccountAction",
        event: "acceptRules",
      };
      OZONE.ajax.requestModule("createaccount/CreateAccount0Module", params, AcceptTOSModule.callbacks.nextClick);
    }
  },
  callbacks: {
    nextClick: function (response: YahooResponse): void {
      if(response.status == "must_accept") {
        document.getElementById("accept-tos-error")!.innerHTML = response.message;
        document.getElementById("accept-tos-error")!.style.display = "block";
        OZONE.dialog.factory.boxcontainer().centerContent();
        return;
      }
      if(!Wikijump.utils.handleError(response)) return;
      const w = new OZONE.dialogs.Dialog();
      w.content = response.body;
      w.show();
    }
  }
};
