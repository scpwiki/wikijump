import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

declare const HTTP_SCHEMA: "http" | "https";

export const CreateAccountModule = {
  vars: {
    formData: null as null | Record<string, string>
  },
  listeners: {
    cancel: function (_event?: Event | null): void {
      window.location.href = `${HTTP_SCHEMA}://${window.location.hostname}`;
    },
    nextClick: function (_event?: Event | null): void {
      CreateAccountModule.vars.formData = OZONE.utils.formToArray("createaccount-form0");

      const params: RequestModuleParameters = {
        ...OZONE.utils.formToArray("createaccount-form0"),
        action: "CreateAccount2Action",
        event: "step0",
      };
      OZONE.ajax.requestModule("createaccount/CreateAccount2Module", params, CreateAccountModule.callbacks.nextClick);
    }
  },
  callbacks: {
    nextClick: function (response: YahooResponse): void {
      if (response.status === "form_errors") {
        let inner = "The data you have submitted contains following errors:" +
          "<ul>";

        const errors = response.formErrors;
        for (const i in errors) {
          inner += `<li>${errors[i]}</li>`;
        }

        inner += "</ul>";

        document.getElementById("ca-reg0-errors")!.style.display = "block";
        document.getElementById("ca-reg0-errors")!.innerHTML = inner;
        return;
      }
      if (response.status == "email_failed") {
        document.getElementById("ca-reg0-errors")!.innerHTML = response.message;
        document.getElementById("ca-reg0-errors")!.style.display = "block";
        return;
      }
      window.location.href = '/auth:newaccount2';
    }
  },
  init: function (): void {
    // 	if form data already exists - fill the forms
    const formData = CreateAccountModule.vars.formData;
    if (formData != null) {
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
    // XXX Get rid of this
    OZONE.dom.onDomReady(function (): void {
      // change links to http://...
      const els = document.getElementsByTagName('a');
      for (let i = 0; i < els.length; i++) {
        els[i].href = els[i].href.replace(/^https/, 'http');
      }
    }, "dummy-ondomready-block");
  }
};

CreateAccountModule.init();
