import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

// From crypto/rsa.js
declare const RSAKey: any;
declare const linebrk: any;
// From crypto/base64.js
declare const hex2b64: any;

export const ASPasswordModule = {
  listeners: {
    save: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        ...OZONE.utils.formToArray("change-password-form"),
        action: "AccountSettingsAction",
        event: "changePassword",
      };
      const rsa = new RSAKey();
      rsa.setPublic(Wikijump.vars.rsakey, "10001");
      params.old_password = linebrk(hex2b64(rsa.encrypt('__' + params.old_password)), 64);
      params.new_password1 = linebrk(hex2b64(rsa.encrypt('__' + params.new_password1)), 64);
      params.new_password2 = linebrk(hex2b64(rsa.encrypt('__' + params.new_password2)), 64);

      OZONE.ajax.requestModule(null, params, ASPasswordModule.callbacks.save);
    }
  },
  callbacks: {
    save: function (response: YahooResponse): void {
      if (response.status == "form_error") {
        const er = document.getElementById("password-error")!;
        er.style.display = "block";
        er.innerHTML = response.message;
        return;
      }
      if (response.status == 'ok') {
        const w = new OZONE.dialogs.SuccessBox();
        w.content = "Your password has been changed.";
        w.show();
        (<HTMLFormElement>document.getElementById("change-password-form")!).reset();
        setTimeout(() => Wikijump.modules.AccountModule.utils.loadModule('am-settings'), 1000);
        return;
      }
    }
  }
};
