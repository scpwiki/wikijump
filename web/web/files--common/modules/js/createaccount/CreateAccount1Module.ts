import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

export const CreateAccount1Module = {
  listeners: {
    cancelClick: function (_event?: Event | null): void {
      OZONE.dialog.cleanAll();
    },
    backClick: function (_event?: Event | null): void {
      OZONE.ajax.requestModule("createaccount/CreateAccount0Module", {}, CreateAccount1Module.callbacks.backClick);
    },
    nextClick: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        action: "CreateAccountAction",
        event: "sendEmailVer"
      };
      OZONE.ajax.requestModule("createaccount/CreateAccount2Module", params, CreateAccount1Module.callbacks.nextClick);
    }
  },
  callbacks: {
    nextClick: function (response: YahooResponse): void {
      if (response.status == "email_failed") {
        document.getElementById("ca-error-block")!.innerHTML = response.message;
        document.getElementById("ca-error-block")!.style.display = "block";
        OZONE.dialog.factory.boxcontainer().centerContent();
        return;
      }
      if (!Wikijump.utils.handleError(response)) { return; }
      const w = new OZONE.dialogs.Dialog();
      w.content = response.body;
      w.show();
    },
    backClick: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const w = new OZONE.dialogs.Dialog();
      w.content = response.body;
      w.show();
    }
  },
  init: function (): void {
    const formData  = Wikijump.modules.CreateAccountModule.vars.formData;
    if (formData == null) {
      alert("Registration flow error.");
      window.location.reload();
      return;
    }
    document.getElementById("ca-field-name")!.innerHTML = formData.name;
    document.getElementById("ca-field-email")!.innerHTML = formData.email;
  }
};
CreateAccount1Module.init();

// YAHOO.util.Event.addListener("next-click", "click", Wikijump.modules.AcceptTOSModule.listeners.nextClick);
