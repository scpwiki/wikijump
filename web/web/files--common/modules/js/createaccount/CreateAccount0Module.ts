import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

// From crypto/rsa.js
declare const RSAKey: any;
declare const linebrk: any;
// From crypto/base64.js
declare const hex2b64: any;

export const CreateAccount0Module = {
  listeners: {
    cancelClick: function (_event?: Event | null): void {
      OZONE.dialog.cleanAll();
    },
    nextClick: function (_event?: Event | null): void {
      Wikijump.modules.CreateAccountModule.vars.formData = OZONE.utils.formToArray("createaccount-form0");

      const params: RequestModuleParameters = {
        ...OZONE.utils.formToArray("createaccount-form0"),
        action: "CreateAccountAction",
        event: "step0",
      };
      const rsa = new RSAKey();
      rsa.setPublic(Wikijump.modules.CreateAccountModule.vars.rsakey, "10001");
      params.email = linebrk(hex2b64(rsa.encrypt(`__${params.email}`)), 64);
      params.password = linebrk(hex2b64(rsa.encrypt(`__${params.password}`)), 64);
      params.password2 = linebrk(hex2b64(rsa.encrypt(`__${params.password2}`)), 64);
      OZONE.ajax.requestModule("createaccount/CreateAccount2Module", params, CreateAccount0Module.callbacks.nextClick);
    }
  },
  callbacks: {
    nextClick: function (response: YahooResponse): void {
      if (response.status == "form_errors") {
        let inner = "The data you have submitted contains following errors:" +
          "<ul>";

        const errors = response.formErrors;
        for (const i in errors) {
          inner += `<li>${errors[i]}</li>`;
        }

        inner += "</ul>";

        document.getElementById("ca-reg0-errors")!.style.display = "block";
        document.getElementById("ca-reg0-errors")!.innerHTML = inner;
        OZONE.dialog.factory.boxcontainer().centerContent();
        return;
      }
      if (response.status == "email_failed") {
        document.getElementById("ca-reg0-errors")!.innerHTML = response.message;
        document.getElementById("ca-reg0-errors")!.style.display = "block";
        OZONE.dialog.factory.boxcontainer().centerContent();
        return;
      }
      if (!Wikijump.utils.handleError(response)) { return; }
      const w = new OZONE.dialogs.Dialog();
      w.content = response.body;
      w.show();
    }
  },
  init: function (): void {
    // 	if form data already exists - fill the forms
    if (Wikijump.modules.CreateAccountModule.vars.formData != null) {
      const formData = Wikijump.modules.CreateAccountModule.vars.formData;
      const form = <HTMLFormElement>(document.getElementById("caform")!);
      (<HTMLInputElement>form.elements.namedItem("name")).value = formData["name"];
      (<HTMLInputElement>form.elements.namedItem("password")).value = formData["password"];
      (<HTMLInputElement>form.elements.namedItem("password2")).value = formData["password2"];
      (<HTMLInputElement>form.elements.namedItem("email")).value = formData["email"];
      (<HTMLInputElement>form.elements.namedItem("captcha")).value = formData["captcha"];
      (<HTMLInputElement>form.elements.namedItem("tos")).checked = true;
      if (formData.language === 'en') {
        (<HTMLInputElement>document.getElementById("new-site-lang-en")!).checked = true;
      } else {
        (<HTMLInputElement>document.getElementById("new-site-lang-pl")!).checked = true;
      }
    }
  }
};

CreateAccount0Module.init();
