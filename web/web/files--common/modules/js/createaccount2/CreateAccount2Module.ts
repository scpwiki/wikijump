import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

declare const HTTP_SCHEMA: "http" | "https";

export const CreateAccount2Module = {
  listeners: {
    cancel: function (_event?: Event | null): void {
      window.location.href = `${HTTP_SCHEMA}://${window.location.hostname}`;
    },
      backClick: function (_event?: Event | null): void {
        OZONE.ajax.requestModule("createaccount/CreateAccount0Module", {}, CreateAccount2Module.callbacks.backClick);
      },
      nextClick: function (_event?: Event | null): void {
        const params: RequestModuleParameters = {
          evcode: (<HTMLInputElement>document.getElementById("ca-evercode")!).value,
          action: "CreateAccount2Action",
          event: "finalize"
        };
        OZONE.ajax.requestModule("Empty", params, CreateAccount2Module.callbacks.nextClick);
      }
  },
  callbacks: {
    nextClick: function (response: YahooResponse): void {
      if (response.status == "invalid_code") {
        document.getElementById("ca-error-block")!.innerHTML = response.message;
        document.getElementById("ca-error-block")!.style.display = "block";
        return;
      }
      if (!Wikijump.utils.handleError(response)) { return; }
      const t2 = new OZONE.dialogs.SuccessBox(); t2.timeout = 10000; t2.content = "New account created!"; t2.show();
      const originalUrl = response.originalUrl;
      if (response.originalUrlForce) {
        setTimeout(function (): void {
          window.location.href = response.originalUrl;
        }, 2000);
      } else {
        setTimeout(function (): void {
          let url = '/auth:newaccount3';
          if (originalUrl) {
            url = `${url}?origUrl=${encodeURIComponent(originalUrl)}`;
          }
          window.location.href = url;
        }, 2000);
      }
    },
    backClick: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const w = new OZONE.dialogs.Dialog();
      w.content = response.body;
      w.show();
    }

  },
  init: function (): void {
    OZONE.dom.onDomReady(function (): void {
      // change links to http://...
      const els = document.getElementsByTagName('a');
      for (let i = 0; i < els.length; i++) {
        els[i].href = els[i].href.replace(/^https/, 'http');
      }
    }, "dummy-ondomready-block");
  }
};
