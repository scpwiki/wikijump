import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

export const ASEmailModule = {
  listeners: {
    next1: function (_event?: Event | null): void {
      const email = (<HTMLInputElement>document.getElementById("ch-email")!).value;

      if (email == null || email == '') {
        document.getElementById("email-change-error")!.innerHTML = "Email must be provided.";
        document.getElementById("email-change-error")!.style.display = "block";
        return;
      }
      if (!email.match(/^[_a-zA-Z0-9-]+(\.[_a-zA-Z0-9-]+)*@[a-zA-Z0-9-]+(\.[a-zA-Z0-9-]+)+$/)) {
        document.getElementById("email-change-error")!.innerHTML = "Valid email must be provided.";
        document.getElementById("email-change-error")!.style.display = "block";
        return;
      }

      const params: RequestModuleParameters = {
        email: email,
        action: "AccountSettingsAction",
        event: "changeEmail1"
      };
      OZONE.ajax.requestModule("account/settings/email/ASChangeEmail2Module", params, ASEmailModule.callbacks.next1);
    },
    next2: function (_event?: Event | null): void {
      const evcode = (<HTMLInputElement>document.getElementById("ch-evercode")!).value;

      const params: RequestModuleParameters = {
        action: "AccountSettingsAction",
        event: "changeEmail2",
        evercode: evcode
      };
      OZONE.ajax.requestModule("account/settings/email/ASChangeEmail3Module", params, ASEmailModule.callbacks.next2);
    }
  },
  callbacks: {
    next1: function (response: YahooResponse): void {
      if (response.status == "form_error") {
        document.getElementById("email-change-error")!.innerHTML = response.message;
        document.getElementById("email-change-error")!.style.display = "block";
        return;
      }
      if (!Wikijump.utils.handleError(response)) { return; }

      document.getElementById("email-change-area")!.innerHTML = response.body;
    },
    next2: function (response: YahooResponse): void {
      if (response.status == "form_error") {
        document.getElementById("email-change-error")!.innerHTML = response.message;
        document.getElementById("email-change-error")!.style.display = "block";
        return;
      }
      if (!Wikijump.utils.handleError(response)) { return; }
      document.getElementById("email-change-area")!.innerHTML = response.body;
      document.getElementById("ech-note")!.style.display = "none";
      document.getElementById("ch-el01")!.style.display = "none";
    }

  }
};
