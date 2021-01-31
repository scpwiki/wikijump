import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
import { Wikirequest } from "wikirequest";
declare const WIKIREQUEST: Wikirequest;
declare type YahooResponse = any;

export const CreateAccount2Module = {
  listeners: {
    cancelClick: function (_event?: Event | null): void {
      window.location.href = "/";
    },
    backClick: function (_event?: Event | null): void {
      OZONE.ajax.requestModule("createaccount/CreateAccount0Module", {}, CreateAccount2Module.callbacks.backClick);
    },
    nextClick: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        evcode: (<HTMLInputElement>document.getElementById("ca-evercode")!).value,
        action: "CreateAccountAction",
        event: "finalize"
      };
      OZONE.ajax.requestModule("createaccount/CreateAccount3Module", params, CreateAccount2Module.callbacks.nextClick);
    }
  },
  callbacks: {
    nextClick: function (response: YahooResponse): void {
      if (response.status == "invalid_code") {
        document.getElementById("ca-error-block")!.innerHTML = response.message;
        document.getElementById("ca-error-block")!.style.display = "block";
        OZONE.dialog.factory.boxcontainer().centerContent();
        return;
      }
      if (!Wikijump.utils.handleError(response)) { return; }

      if (!WIKIREQUEST.createAccountSkipCongrats) {
        const w = new OZONE.dialogs.Dialog();
        w.content = response.body;
        w.show();
      } else {
        window.location.reload();
      }
    },
    backClick: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const w = new OZONE.dialogs.Dialog();
      w.content = response.body;
      w.show();
    }

  }
};

// YAHOO.util.Event.addListener("next-click", "click", Wikijump.modules.AcceptTOSModule.listeners.nextClick);
