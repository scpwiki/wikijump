import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
import { Category } from "./ManageSiteModule";
declare const YAHOO: any;
declare type YahooResponse = any;

export const ManageSiteTemplatesModule = {
  vars: {
    currentCategory: null as null | Category
  },
  listeners: {
    categoryChange: function (_event?: Event | null): void {
      // update template info
      const categoryId = parseInt((<HTMLSelectElement>document.getElementById("sm-appearance-cats")!).value);
      const category = Wikijump.modules.ManageSiteModule.utils.getCategoryById(categoryId);
      ManageSiteTemplatesModule.vars.currentCategory = category;

      const value = category.template_id;
      if (value == null) {
        (<HTMLSelectElement>document.getElementById("sm-templates-list")!).value = "";
      } else {
        (<HTMLSelectElement>document.getElementById("sm-templates-list")!).value = value.toString();
      }
      ManageSiteTemplatesModule.utils.updateTemplatePreview();
    },
    templateChange: function (_event?: Event | null): void {
      // save changes to the array
      const categoryId = parseInt((<HTMLSelectElement>document.getElementById("sm-appearance-cats")!).value);
      const category = Wikijump.modules.ManageSiteModule.utils.getCategoryById(categoryId);
      // @ts-expect-error What is this?
      if (this.value == "") {
        category.template_id = null;
      } else {
        // @ts-expect-error What is this?
        category.template_id = this.value;
      }
      ManageSiteTemplatesModule.utils.updateTemplatePreview();
    },
    cancel: function (_event?: Event | null): void {
      Wikijump.modules.ManageSiteModule.utils.loadModule('sm-welcome');
    },
    save: function (_event?: Event | null): void {
      // ok, do it the easy way: serialize categories using the JSON method
      const categories = Wikijump.modules.ManageSiteModule.vars.categories;
      const serialized = JSON.stringify(categories);
      const params: RequestModuleParameters = {
        categories: serialized,
        action: "ManageSiteAction",
        event: "saveTemplates"
      };
      OZONE.ajax.requestModule("Empty", params, ManageSiteTemplatesModule.callbacks.save);
      const w = new OZONE.dialogs.WaitBox();
      w.content = "Saving changes...";
      w.show();
    }
  },
  callbacks: {
    cancel: function (response: YahooResponse): void {
      OZONE.utils.setInnerHTMLContent("site-manager", response.body);
    },
    save: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const w = new OZONE.dialogs.SuccessBox();
      w.content = "Changes saved.";
      w.show();
    }
  },
  utils: {
    updateTemplatePreview: function (): void {
      // apart from just updating the preview also show/hide extra textarea
      // for custom licenses
      const templateId = (<HTMLSelectElement>document.getElementById("sm-templates-list")!).value;

      // now enable or disable preview
      // first hide all previews

      const div = document.getElementById("sm-template-preview")!;

      if (templateId == "") {
        div.style.display = "none";
        return;
      } else {
        div.style.display = "block";
      }
      const pres = div.getElementsByTagName("div");
      for (let i = 0; i < pres.length; i++) {
        pres[i].style.display = "none";
      }
      // now show the chosen one
      const pre = document.getElementById(`sm-template-preview-${templateId}`)!;
      pre.style.display = "block";
    }
  },
  init: function (): void {
    YAHOO.util.Event.addListener("sm-template-cats", "change", ManageSiteTemplatesModule.listeners.categoryChange);
    YAHOO.util.Event.addListener("sm-templates-list", "change", ManageSiteTemplatesModule.listeners.templateChange);

    YAHOO.util.Event.addListener("sm-templates-cancel", "click", ManageSiteTemplatesModule.listeners.cancel);
    YAHOO.util.Event.addListener("sm-templates-save", "click", ManageSiteTemplatesModule.listeners.save);
    // init categories info
    if (document.getElementById("sm-template-cats")!) {
      ManageSiteTemplatesModule.listeners.categoryChange(null);
    }
  }
};

ManageSiteTemplatesModule.init();
