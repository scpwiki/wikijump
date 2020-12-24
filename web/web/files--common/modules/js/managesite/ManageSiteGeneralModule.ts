import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
import { Wikirequest } from "wikirequest";
declare const WIKIREQUEST: Wikirequest;
declare const YAHOO: any;
declare type YahooResponse = any;

export const ManageSiteGeneralModule = {
  listeners: {
    save: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        ...OZONE.utils.formToArray("sm-general-form"),
        action: "ManageSiteAction",
        event: "saveGeneral",
      };
      OZONE.ajax.requestModule("Empty", params, ManageSiteGeneralModule.callbacks.save);
      const w = new OZONE.dialogs.WaitBox();
      w.content = "Saving changes...";
      w.show();
    },
    cancel: function (_event?: Event | null): void {
      Wikijump.modules.ManageSiteModule.utils.loadModule('sm-welcome');
    }
  },
  callbacks: {
    save: function (response: YahooResponse): void {
      if (response.status == "form_errors") {
        OZONE.dialog.cleanAll();
        let inner = "The data you have submitted contains following errors:" +
          "<ul>";

        const errors = response.formErrors;
        for (const i in errors) {
          inner += "<li>" + errors[i] + "</li>";
        }
        inner += "</ul>";
        document.getElementById("sm-general-errorblock")!.innerHTML = inner;
        document.getElementById("sm-general-errorblock")!.style.display = "block";
        return;
      }
      if (!Wikijump.utils.handleError(response)) { return; }
      document.getElementById("sm-general-errorblock")!.style.display = "none";
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
    YAHOO.util.Event.addListener("sm-general-cancel", "click", ManageSiteGeneralModule.listeners.cancel);
    YAHOO.util.Event.addListener("sm-general-save", "click", ManageSiteGeneralModule.listeners.save);
    new OZONE.forms.lengthLimiter("site-description-field", "site-description-field-left", 300);

    // attach the autocomplete thing
    const myDataSource = new YAHOO.widget.DS_XHR("/quickmodule.php", ['pages', 'unix_name', 'title']);
    myDataSource.scriptQueryParam = "q";
    myDataSource.scriptQueryAppend = "s=" + WIKIREQUEST.info.siteId + "&module=PageLookupQModule";

    const myAutoComp = new YAHOO.widget.AutoComplete("sm-general-start", "sm-general-start-list", myDataSource);
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

    myAutoComp.autoHighlight = false;
    myAutoComp.minQueryLength = 2;
    myAutoComp.queryDelay = 0.5;
  }
};

ManageSiteGeneralModule.init();
