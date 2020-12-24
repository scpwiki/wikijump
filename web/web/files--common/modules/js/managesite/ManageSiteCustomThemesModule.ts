import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
import { Wikirequest } from "wikirequest";
declare const WIKIREQUEST: Wikirequest;
declare const YAHOO: any;
declare type YahooResponse = any;

export const ManageSiteCustomThemesModule = {
  vars: {
    currentThemeId: null as null | number,
    editThemeId: null as null | number
  },
  listeners: {
    editTheme: function (_event: Event | null, themeId: number): void {
      const params: RequestModuleParameters = {};
      if (themeId) {
        params.themeId = themeId;
      }
      ManageSiteCustomThemesModule.vars.editThemeId = themeId;
      OZONE.ajax.requestModule("managesite/ManageSiteEditCustomThemeModule", params, ManageSiteCustomThemesModule.callbacks.editTheme);
    },
    importCss: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        pageName: (<HTMLInputElement>document.getElementById("sm-cssimport-input")!).value,
        action: "ManageSiteAction",
        event: "importCss",
      };
      if (params.pageName === "") {
        document.getElementById("cssimport-error")!.innerHTML = "In order to import CSS you should first give a non-empty page name.";
        document.getElementById("cssimport-error")!.style.display = "block";
        return;
      }
      OZONE.ajax.requestModule(null, params, ManageSiteCustomThemesModule.callbacks.importCss);
    },
    cancelEditTheme: function (_event?: Event | null): void {
      document.getElementById("edit-theme-box")!.innerHTML = "";
    },
    saveTheme: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        ...OZONE.utils.formToArray("sm-edit-theme-form"),
        action: "ManageSiteAction",
        event: "customThemeSave",
      };
      if (ManageSiteCustomThemesModule.vars.editThemeId) {
        params.themeId = ManageSiteCustomThemesModule.vars.editThemeId;
      }
      OZONE.ajax.requestModule(null, params, ManageSiteCustomThemesModule.callbacks.saveTheme);
      const w = new OZONE.dialogs.WaitBox();
      w.content = "Saving theme...";
      w.show();
    },
    deleteTheme: function (_event: Event | null, themeId: number): void {
      ManageSiteCustomThemesModule.vars.currentThemeId = themeId;
      const w = new OZONE.dialogs.ConfirmationDialog();
      w.content = "Are you sure you want to delete this theme?";
      w.buttons = ["cancel", "yes, delete"];
      w.addButtonListener("cancel", w.close);
      w.addButtonListener("yes, delete", () => {
        ManageSiteCustomThemesModule.listeners.deleteTheme2(
          null, ManageSiteCustomThemesModule.vars.currentThemeId
        );
      });
      w.show();
    },
    deleteTheme2: function (_event: Event | null, themeId: number | null): void {
      const params: RequestModuleParameters = {
        action: "ManageSiteAction",
        event: "customThemeDelete",
      };
      if (themeId) { params.themeId = themeId; }
      OZONE.ajax.requestModule(null, params, ManageSiteCustomThemesModule.callbacks.deleteTheme);
    }
  },
  callbacks: {
    editTheme: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      document.getElementById("edit-theme-box")!.innerHTML = response.body;

      // attach autocomplete
      // attach the autocomplete thing
      const myDataSource = new YAHOO.widget.DS_XHR("/quickmodule.php", ['pages', 'unix_name', 'title']);
      myDataSource.scriptQueryParam = "q";
      myDataSource.scriptQueryAppend = `s=${WIKIREQUEST.info.siteId}&module=PageLookupQModule`;

      const myAutoComp = new YAHOO.widget.AutoComplete("sm-cssimport-input", "sm-cssimport-input-list", myDataSource);
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
      myAutoComp.minQueryLength = 2;
      myAutoComp.queryDelay = 0.5;
    },
    importCss: function (response: YahooResponse): void {
      if (response.status == "form_error") {
        document.getElementById("cssimport-error")!.innerHTML = response.message;
        document.getElementById("cssimport-error")!.style.display = "block";
        return;
      }
      document.getElementById("cssimport-error")!.style.display = "none";
      if (!Wikijump.utils.handleError(response)) { return; }
      (<HTMLTextAreaElement>document.getElementById("sm-csscode")!).value = response.code;
    },
    saveTheme: function (response: YahooResponse): void {
      if (response.status == "form_error") {
        document.getElementById("edit-theme-error")!.innerHTML = response.message;
        document.getElementById("edit-theme-error")!.style.display = "block";
        OZONE.visuals.scrollTo("edit-theme-error");
        OZONE.dialog.cleanAll();
        return;
      }
      document.getElementById("edit-theme-error")!.style.display = "none";
      if (!Wikijump.utils.handleError(response)) { return; }
      const w = new OZONE.dialogs.SuccessBox();
      w.content = "Theme saved.";
      w.show();
      setTimeout(() => Wikijump.modules.ManageSiteModule.utils.loadModule("sm-customthemes"), 1000);
    },
    deleteTheme: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      Wikijump.modules.ManageSiteModule.utils.loadModule("sm-customthemes");
      OZONE.dialog.cleanAll();
    }
  }
};
