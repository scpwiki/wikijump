import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare const YAHOO: any;
declare type YahooResponse = any;

export const ManageSiteDomainModule = {
  listeners: {
    save: function (event: Event): void {
      const domain = (<HTMLInputElement>document.getElementById("sm-domain-field")!).value;

      const redirects = [];
      const container = document.getElementById("sm-redirects-box")!;
      // count them!
      const inputs = container.getElementsByTagName('input');
      for (let i = 0; i < inputs.length; i++) {
        const redirUrl = inputs[i].value;
        if (redirUrl) {
          if (!redirUrl.match(/^[a-z0-9-]+(\.[a-z0-9-]+)+$/i)) {
            document.getElementById("sm-domain-error")!.innerHTML = `"${redirUrl}" is not a valid domain. Please correct it and save again.`;
            document.getElementById("sm-domain-error")!.style.display = "block";
            return;
          }
        }
        redirects.push(redirUrl);
      }

      const redirectsString = redirects.join(';');

      const params: RequestModuleParameters = {
        redirects: redirectsString,
        domain: domain,
        action: "ManageSiteAction",
        event: "saveDomain"
      };
      OZONE.ajax.requestModule("Empty", params, ManageSiteDomainModule.callbacks.save);
      const w = new OZONE.dialogs.WaitBox();
      w.content = "Saving changes...";
      w.show();
      YAHOO.util.Event.preventDefault(event);
    },
    cancel: function (_event?: Event | null): void {
      Wikijump.modules.ManageSiteModule.utils.loadModule('sm-welcome');
    },
    clear: function (_event?: Event | null): void {
      (<HTMLInputElement>document.getElementById("sm-domain-field")!).value = "";
      document.getElementById("sm-redirects-box")!.innerHTML = '';
      ManageSiteDomainModule.listeners.addRedirect();
    },
    addRedirect: function (_event?: Event | null): void {
      const container = document.getElementById("sm-redirects-box")!;
      // count them!
      const divs = container.getElementsByTagName('div');
      if (divs.length >= 10) {
        alert("Sorry, you can have only up to 10 redirects defined");
        return;
      }
      const inn = document.getElementById("sm-redirect-template")!.innerHTML;
      const div = document.createElement('div');
      div.innerHTML = inn;
      container.appendChild(div);
    },
    removeRedirect: function (event: Event): void {
      let el = YAHOO.util.Event.getTarget(event);

      while (el && el.tagName && el.tagName !== 'DIV') {
        el = el.parentNode;
      }
      el.parentNode.removeChild(el);
    }
  },
  callbacks: {
    save: function (response: YahooResponse): void {
      if (response.status == "form_errors") {
        document.getElementById("sm-domain-errorblock")!.innerHTML = response.message;
        document.getElementById("sm-domain-errorblock")!.style.display = "block";
        return;
      }
      if (!Wikijump.utils.handleError(response)) { return; }
      const w = new OZONE.dialogs.SuccessBox();
      w.content = "Changes saved.";
      w.show();
    },
    cancel: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      OZONE.utils.setInnerHTMLContent("site-manager", response.body);
    }
  },
  init: function (): void {
    YAHOO.util.Event.addListener("sm-domain-cancel", "click", ManageSiteDomainModule.listeners.cancel);
    YAHOO.util.Event.addListener("sm-domain-clear", "click", ManageSiteDomainModule.listeners.clear);
    YAHOO.util.Event.addListener("sm-domain-save", "click", ManageSiteDomainModule.listeners.save);
    YAHOO.util.Event.addListener("sm-domain-form", "submit", ManageSiteDomainModule.listeners.save);
  }
};

ManageSiteDomainModule.init();
