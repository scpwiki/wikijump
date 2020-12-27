import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare const YAHOO: any;
declare type YahooResponse = any;

// From crypto/rsa.js
declare const RSAKey: any;
declare const linebrk: any;
// From crypto/base64.js
declare const hex2b64: any;

export const LoginModule3 = {
  listeners: {
    loginClick: function (event: Event): void {
      YAHOO.util.Event.preventDefault(event);
      const params: RequestModuleParameters = {
        ...OZONE.utils.formToArray("login-form"),
        action: "LoginAction",
        event: "login",
      };
      // pre-check:
      const welcome = OZONE.utils.getCookie('welcome');
      if ((welcome == null && params.name == '') || params.password == '') {
        const message = "Please fill the login form.";
        document.getElementById('loginerror')!.innerHTML = message;
        document.getElementById("login-head")!.style.display = "none";
        document.getElementById('loginerror')!.style.display = "block";
        return;
      }

      if (welcome) {
        params.welcome = welcome;
      }

      document.getElementById("login-buttons")!.style.display = "none";
      document.getElementById("login-progress")!.style.display = "block";

      const rsa = new RSAKey();
      rsa.setPublic(Wikijump.vars.rsakey, "10001");
      params.name = linebrk(hex2b64(rsa.encrypt(Wikijump.vars.loginSeed! + params.loginName)), 64);
      params.password = linebrk(hex2b64(rsa.encrypt(Wikijump.vars.loginSeed! + params.password)), 64);
      OZONE.ajax.requestModule(null, params, LoginModule3.callbacks.loginClick);
    },
    cancel: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        action: "LoginAction",
        event: "loginCancel"
      };
      OZONE.ajax.requestModule(null, params, LoginModule3.callbacks.cancel);
    },
    namePress: function (event: Event): void {
      const chcode = YAHOO.util.Event.getCharCode(event);
      if ((chcode == 13) && (<HTMLInputElement>document.getElementById('login-form-name')!).value.length > 0) {
        YAHOO.util.Event.stopEvent(event);
        document.getElementById('login-form-password')!.focus();
      }
    }
  },
  callbacks: {
    loginClick: function (response: YahooResponse): void {
      if (response.status == 'login_invalid') {
        document.getElementById("login-head")!.style.display = "none";
        document.getElementById("loginerror")!.innerHTML = response.message;
        document.getElementById("loginerror")!.style.display = "block";

        document.getElementById("login-buttons")!.style.display = "block";
        document.getElementById("login-progress")!.style.display = "none";
        return;
      }
      if (!Wikijump.utils.handleError(response)) { return; }
      setTimeout(() => top.location.href="' + Wikijump.vars.backUrl + '", 1000);
    },
    cancel: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      OZONE.dialog.cleanAll();
    }
  },
  init: function (): void {
    if (document.getElementById('login-form-name')! && (<HTMLInputElement>document.getElementById('login-form-name')!).type == "text") {
      document.getElementById('login-form-name')!.focus();
      YAHOO.util.Event.addListener(document.getElementById('login-form-name')!, 'keypress', LoginModule3.listeners.namePress);
    } else {
      document.getElementById('login-form-password')!.focus();
    }
    YAHOO.util.Event.addListener("login-form", 'submit', LoginModule3.listeners.loginClick);
  }
};

LoginModule3.init();
