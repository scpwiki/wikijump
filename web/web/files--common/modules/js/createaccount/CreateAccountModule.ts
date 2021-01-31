import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
declare type YahooResponse = any;

export const CreateAccountModule = {
  vars: {
    // XXX crypto/rsa.js
    rsakey: null as any,
    formData: null as null | Record<string, string>
  },
  listeners: {
    createClick: function (_event?: Event | null): void {
      OZONE.ajax.requestModule("createaccount/CreateAccount0Module", {}, CreateAccountModule.callbacks.createClick);
    },
    cancel: function (_event?: Event | null): void {
      const params = {
        action: "CreateAccountAction",
        event: "cancel"
      };
      OZONE.ajax.requestModule(null, params, CreateAccountModule.callbacks.cancel);
    }
  },
  callbacks: {
    createClick: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const w = new OZONE.dialogs.Dialog();
      w.content = response.body;
      w.show();

      // store seed and key
      CreateAccountModule.vars.rsakey = response.key;
    },
    cancel: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      OZONE.dialog.cleanAll();
    }
  }
};
