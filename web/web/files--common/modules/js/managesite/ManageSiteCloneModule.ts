import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;
declare const HTTP_SCHEMA: string;
declare const URL_DOMAIN: string;

export const ManageSiteCloneModule = {
  vars: {
    unixname: null as null | string
  },
  listeners: {

    cloneSite: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        ...OZONE.utils.formToArray("clone-site-form"),
        action: "ManageSiteCloneAction",
        event: "cloneSite",
      };
      OZONE.ajax.requestModule("managesite/ManageSiteClone2Module", params, ManageSiteCloneModule.callbacks.cloneSite);
      const w = new OZONE.dialogs.WaitBox();
      w.content = "Cloning site...";
      w.show();
      ManageSiteCloneModule.vars.unixname = <string>params.unixname;
    },
    cancel: function (_event?: Event | null): void {
      Wikijump.modules.ManageSiteModule.utils.loadModule('sm-welcome');
    },
    goToTheSite: function (_event?: Event | null): void {
      window.location.href = `${HTTP_SCHEMA}://${ManageSiteCloneModule.vars.unixname}.${URL_DOMAIN}`;
    }
  },
  callbacks: {
    cloneSite: function (response: YahooResponse): void {
      if (response.status == "form_errors") {
        OZONE.dialog.cleanAll();
        let inner = "The data you have submitted contains following errors:" +
          "<ul>";

        const errors = response.formErrors;
        for (const i in errors) {
          inner += "<li>" + errors[i] + "</li>";
        }
        inner += "</ul>";
        document.getElementById("clone-site-form-errors")!.innerHTML = inner;
        document.getElementById("clone-site-form-errors")!.style.display = "block";
        return;
      }
      if (!Wikijump.utils.handleError(response)) { return; }
      document.getElementById("clone-site-form-errors")!.innerHTML = '';
      document.getElementById("sm-clone-block")!.innerHTML = response.body;

      OZONE.dialog.cleanAll();
    }
  },
};
