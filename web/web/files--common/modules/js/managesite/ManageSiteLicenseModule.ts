import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
import { Category } from "./ManageSiteModule";
declare const YAHOO: any;
declare type YahooResponse = any;

export const ManageSiteLicenseModule = {
  vars: {
    currentCategory: null as null | Category,
    limiter: null as null | InstanceType<typeof OZONE.forms.lengthLimiter>
  },
  listeners: {
    categoryChange: function (_event?: Event | null): void {
      // update license info
      const categoryId = parseInt((<HTMLSelectElement>document.getElementById("sm-license-cats")!).value);
      const category = Wikijump.modules.ManageSiteModule.utils.getCategoryById(categoryId);
      ManageSiteLicenseModule.vars.currentCategory = category;
      // check if has a individual license
      if (category.name == "_default") {
        document.getElementById("sm-license-noind")!.style.display = "none";
        document.getElementById("sm-license-list")!.style.display = "";
      } else {
        document.getElementById("sm-license-noind")!.style.display = "block";
        if (category.license_default) {
          (<HTMLInputElement>document.getElementById("sm-license-noin")!).checked = true;
          document.getElementById("sm-license-list")!.style.display = "none";
        } else {
          (<HTMLInputElement>document.getElementById("sm-license-noin")!).checked = false;
          document.getElementById("sm-license-list")!.style.display = "";
        }
      }

      (<HTMLSelectElement>document.getElementById("sm-license-lic")!).value = category.license_id.toString();
      ManageSiteLicenseModule.utils.updateLicensePreview();
    },
    indClick: function (_event?: Event | null): void {
      const categoryId = parseInt((<HTMLSelectElement>document.getElementById("sm-license-cats")!).value);
      const category = Wikijump.modules.ManageSiteModule.utils.getCategoryById(categoryId);

      if ((<HTMLInputElement>document.getElementById("sm-license-noin")!).checked == true) {
        document.getElementById("sm-license-list")!.style.display = "none";
        category.license_default = true;
      } else {
        document.getElementById("sm-license-list")!.style.display = "";
        category.license_default = false;
      }
      ManageSiteLicenseModule.utils.updateLicensePreview();
    },
    licenseChange: function (_event?: Event | null): void {
      // save changes to the array
      const categoryId = parseInt((<HTMLSelectElement>document.getElementById("sm-license-cats")!).value);
      const category = Wikijump.modules.ManageSiteModule.utils.getCategoryById(categoryId);
      // @ts-expect-error What is this?
      category.license_id = this.value;
      ManageSiteLicenseModule.utils.updateLicensePreview();
    },
    otherDescriptionChange: function (_event?: Event | null): void {
      const category = ManageSiteLicenseModule.vars.currentCategory!;
      let text = (<HTMLTextAreaElement>document.getElementById("sm-other-license-text")!).value;
      category.license_other = text;

      // also update the preview...
      const licenseId = (<HTMLSelectElement>document.getElementById("sm-license-lic")!).value;
      const lid = `sm-prev-license-${licenseId}`;
      const prev = document.getElementById(lid)!;
      text = text.split("&").join("&amp;").split("<").join("&lt;").split(">").join("&gt;");
      // now reenable some tags, i.event.: "a", "img" and "br"
      text = text.replace(/&lt;a href="(.*?)"&gt;(.*?)&lt;\/a&gt;/g, '<a href="$1">$2</a>');
      text = text.replace(/&lt;img src="(.*?)"(?: alt="(.*?)")?(?: )*(?:\/)?&gt;/g, '<img src="$1" alt="$2"/>');
      text = text.replace(/&lt;br(\/)?&gt;/g, '<br/>');
      text = text.replace(/&lt;strong&gt;(.*?)&lt;\/strong&gt;/g, '<strong>$1</strong>');
      text = text.replace(/&lt;em&gt;(.*?)&lt;\/em&gt;/g, '<em>$1</em>');

      prev.innerHTML = text;
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
        event: "saveLicense"
      };
      OZONE.ajax.requestModule("Empty", params, ManageSiteLicenseModule.callbacks.save);
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
      w.content = "Changes have been saved";
      w.show();
    }
  },
  utils: {
    updateLicensePreview: function (): void {
      // apart from just updating the preview also show/hide extra textarea
      // for custom licenses
      let licenseId: number;
      const category = ManageSiteLicenseModule.vars.currentCategory!;
      if ((<HTMLInputElement>document.getElementById("sm-license-noin")!).checked == true && category.name != "_default") {
        // get theme_id for the category _default
        const defCategory = Wikijump.modules.ManageSiteModule.utils.getCategoryByName("_default");
        licenseId = defCategory.license_id;
      } else {
        licenseId = parseInt((<HTMLSelectElement>document.getElementById("sm-license-lic")!).value);
      }

      // const us assume that "other" has id = 1. bleeeeh

      if (licenseId == 1) {
        document.getElementById("sm-other-license")!.style.display = "block";
        // fill it with contents
        if (category.name == '_default' || !(<HTMLInputElement>document.getElementById("sm-license-noin")!).checked) {
          (<HTMLTextAreaElement>document.getElementById("sm-other-license-text")!).value = category.license_other;
        } else {
          const defCategory = Wikijump.modules.ManageSiteModule.utils.getCategoryByName("_default");
          (<HTMLTextAreaElement>document.getElementById("sm-other-license-text")!).value = defCategory.license_other;
        }
        ManageSiteLicenseModule.listeners.otherDescriptionChange();
        ManageSiteLicenseModule.vars.limiter!.keyListener();
      } else {
        document.getElementById("sm-other-license")!.style.display = "none";
      }

      // now enable or disable preview
      // first hide all previews
      const div = document.getElementById("sm-license-preview")!;
      const pres = div.getElementsByTagName("div");
      for (let i = 0; i < pres.length; i++) {
        pres[i].style.display = "none";
      }
      // now show the chosen one
      const pre = document.getElementById("sm-prev-license-" + licenseId)!;
      pre.style.display = "block";
    }
  },
  init: function (): void {
    YAHOO.util.Event.addListener("sm-license-cats", "change", ManageSiteLicenseModule.listeners.categoryChange);
    YAHOO.util.Event.addListener("sm-license-lic", "change", ManageSiteLicenseModule.listeners.licenseChange);
    YAHOO.util.Event.addListener("sm-license-noind", "click", ManageSiteLicenseModule.listeners.indClick);
    YAHOO.util.Event.addListener("sm-other-license-text", "keyup", ManageSiteLicenseModule.listeners.otherDescriptionChange);

    YAHOO.util.Event.addListener("sm-license-cancel", "click", ManageSiteLicenseModule.listeners.cancel);
    YAHOO.util.Event.addListener("sm-license-save", "click", ManageSiteLicenseModule.listeners.save);
    // init categories info

    const limiter = new OZONE.forms.lengthLimiter("sm-other-license-text", "sm-other-license-text-left", 300);
    ManageSiteLicenseModule.vars.limiter = limiter;

    ManageSiteLicenseModule.listeners.categoryChange(null);
  }
};

ManageSiteLicenseModule.init();
