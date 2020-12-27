import { compressTight } from "compress-tag";

import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare const YAHOO: any;
declare type YahooResponse = any;

declare const HTTP_SCHEMA: 'http' | 'https';
declare const URL_DOMAIN: string;
declare const URL_HOST: string;

export const LoginModule = {
  listeners: {
    loginClick: function (event: Event): void {
      YAHOO.util.Event.stopEvent(event);

      const params: RequestModuleParameters = {
        ...OZONE.utils.formToArray("login-form"),
        action: "Login2Action",
        event: "login",
      };
      // pre-check:
      const welcome = OZONE.utils.getCookie('welcome');
      if ((welcome == null && params.name == '') || params.password == '') {
        const message = "Please fill the login form.";
        document.getElementById('loginerror')!.innerHTML = message;
        // document.getElementById("login-head")!.style.display = "none";
        document.getElementById('loginerror')!.style.display = "block";
        return;
      }
      if (welcome) {
        params.welcome = welcome;
      }

      OZONE.ajax.requestModule(null, params, LoginModule.callbacks.loginClick);
    },
    switchUser: function (_event?: Event | null): void {
      setCookie('welcome', null, -100000, '/', '.' + URL_DOMAIN);
      setCookie('welcome', null, -100000, '/');
      window.location.reload();
    },
    cancel: function (_event?: Event | null): void {
      const url = getQueryString('origUrl', HTTP_SCHEMA + "://" + URL_HOST);
      window.location.href = url;
    },
    namePress: function (event: Event): void {
      const chcode = YAHOO.util.Event.getCharCode(event);
      if ((chcode == 13 || chcode == 9) && (<HTMLInputElement>document.getElementById('login-form-name')!).value.length > 0) {
        YAHOO.util.Event.preventDefault(event);
        document.getElementById('login-form-password')!.focus();
      }
    }
  },
  callbacks: {
    loginClick: function (response: YahooResponse): void {
      if (response.status == 'login_invalid') {
        document.getElementById("loginerror")!.innerHTML = response.message;
        document.getElementById("loginerror")!.style.display = "block";
        return;
      }

      if (!Wikijump.utils.handleError(response)) { return; }
      const w = new OZONE.dialogs.WaitBox();
      w.content = "Logging in...";
      w.show();
      const originalUrl = response.originalUrl;
      if (originalUrl) {
        window.location.href = originalUrl;
      } else {
        window.location.href = `${HTTP_SCHEMA}://${window.location.host}`;
      }
    },
    cancel: function (_response: YahooResponse): void {
      window.location.href = `${HTTP_SCHEMA}://${window.location.host}`;
    }
  },
  init: function (): void {
    if (document.getElementById('login-form-name')!) {
      document.getElementById('login-form-name')!.focus();
      YAHOO.util.Event.addListener(document.getElementById('login-form-name')!, 'keypress', LoginModule.listeners.namePress);
    } else {
      document.getElementById('login-form-password')!.focus();
    }

    OZONE.dom.onDomReady(function (): void {
      // change links to http://...
      const els = document.getElementsByTagName('a');
      for (let i = 0; i < els.length; i++) {
        els[i].href = els[i].href.replace(/^https/, 'http');
      }
    }, "dummy-ondomready-block");
  }
};

setTimeout(function (): void { LoginModule.init(); }, 100);

function getQueryString (key: string, default_?: string | null): string {
  if (default_ == null) {
    default_ = "";
  }
  key = key.replace(/[[]/, "\\[").replace(/[\]]/, "\\]");
  const regex = new RegExp("[\\?&]" + key + "=([^&#]*)");
  const qs = regex.exec(window.location.href);
  if (qs == null) {
    return default_;
  }
  return decodeURIComponent(qs[1]);
}

function setCookie (
  name: string,
  value: string | null,
  expires: number,
  path: string,
  domain?: string,
  secure?: boolean
): void {
  const today = new Date();
  today.setTime(today.getTime());
  if (expires) {
    expires = expires * 1000 * 60 * 60 * 24;
  }
  const expires_date = new Date(today.getTime() + (expires));

  const ck = compressTight`
    ${name}=${value == null ? value : escape(value)}
    ${expires ? `;expires=${expires_date.toUTCString()}` : ''}
    ${path ? `;path=${path}` : ''}
    ${domain ? `;domain=${domain}` : ''}
    ${secure ? ';secure' : ''}
  `;
  document.cookie = ck;
}
