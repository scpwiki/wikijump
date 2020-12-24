import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
import { Wikirequest } from "wikirequest";
declare const WIKIREQUEST: Wikirequest;
declare const YAHOO: any;
declare type YahooResponse = any;

export const ManageSiteOpenIDModule = {
  vars: {
    randoms: {} as { [id: number]: boolean },
    services: null as null | { pattern: string; server: string }[]
  },
  listeners: {
    addEntry: function (_event?: Event | null): void {
      let cont = document.getElementById("sm-openid-templateform")!.innerHTML;
      let rand: number;
      // Generate a new random number that has not already been used this
      // session
      do {
        rand = Math.ceil(Math.random() * 10000) + 1000;
      } while (ManageSiteOpenIDModule.vars.randoms[rand] == true);
      ManageSiteOpenIDModule.vars.randoms[rand] = true;

      cont = cont.replace(/RAND/g, rand.toString());

      const div = document.createElement('div');
      div.id = `sm-openid-entry-${rand}`;
      div.innerHTML = cont;
      document.getElementById("sm-openid-idblock")!.appendChild(div);

      const myDataSource = new YAHOO.widget.DS_XHR("/quickmodule.php", ['pages', 'unix_name', 'title']);
      myDataSource.scriptQueryParam = "q";
      myDataSource.scriptQueryAppend = `s=${WIKIREQUEST.info.siteId}&module=PageLookupQModule`;

      const myAutoComp = new YAHOO.widget.AutoComplete(`sm-openid-params-${rand}`, `sm-openid-params-list-${rand}`, myDataSource);
      // @ts-expect-error Autocomp
      myAutoComp.formatResult = function (aResultItem, _sQuery): string {
        const title = aResultItem[1];
        const unixName = aResultItem[0];
        if (unixName != null) {
          return `<div style="font-size: 100%">${unixName}</div><div style="font-size: 80%;">(${title})</div>`;
        } else {
          return "";
        }
      };

      myAutoComp.forceSelection = true;
      myAutoComp.autoHighlight = true;
      myAutoComp.minQueryLength = 2;
      myAutoComp.queryDelay = 0.5;

      OZONE.visuals.scrollTo(div.id);
    },
    deleteEntry: function (_event: Event | null, id: number): void {
      const el = document.getElementById(`sm-openid-entry-${id}`)!;
      if (el) {
        el.parentNode!.removeChild(el);
      }
    },
    onIdentityChange: function (_event: Event | null, id: number): void {
      const url = (<HTMLInputElement>document.getElementById(`sm-openid-urlid-${id}`)!).value;

      // check if URL matches any of the patterns...
      const os = ManageSiteOpenIDModule.vars.services;
      let pattern = null;
      let reg = null;

      for (const i in os!) {
        pattern = os[i].pattern;
        reg = new RegExp(pattern);
        if (reg.test(url)) {
          (<HTMLInputElement>document.getElementById(`sm-openid-urlserver-${id}`)!).value = os[i].server;
        }
      }
    },
    save: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        action: "ManageSiteOpenIDAction",
        event: "saveOpenID",
        enableOpenID: (<HTMLInputElement>document.getElementById("sm-openid-enable")!).checked
      };

      const vals = [];
      vals[0] = OZONE.utils.formToArray("sm-openid-form-0");

      const forms = document.getElementById("sm-openid-idblock")!.getElementsByTagName("form");
      for (let i = 0; i < forms.length; i++) {
        vals.push(OZONE.utils.formToArray(forms[i].id));
      }

      params.vals = JSON.stringify(vals);
      OZONE.ajax.requestModule(null, params, ManageSiteOpenIDModule.callbacks.save);

      const w = new OZONE.dialogs.WaitBox();
      w.content = "Saving changes...";
      w.show();
    }
  },
  callbacks: {
    save: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const w = new OZONE.dialogs.SuccessBox();
      w.content = "Changes saved";
      w.show();
    }
  },
  init: function (): void {
    const os = document.getElementById("sm-openid-patterns")!.innerHTML;
    ManageSiteOpenIDModule.vars.services = JSON.parse(os);
  }
};

ManageSiteOpenIDModule.init();
