import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare const YAHOO: any;
declare type YahooResponse = any;

export const ManageSiteAppearanceModule = {
  vars: {
  },
  listeners: {
    categoryChange: function (_event?: Event | null): void {
      // update theme info
      const categoryId = parseInt((<HTMLSelectElement>document.getElementById("sm-appearance-cats")!).value);
      const category = Wikijump.modules.ManageSiteModule.utils.getCategoryById(categoryId);
      // check if has a individual theme
      ManageSiteAppearanceModule.utils.hideVariants();
      if (category.name == "_default") {
        document.getElementById("sm-appearance-noind")!.style.display = "none";
        document.getElementById("sm-appearance-theme")!.style.display = "block";
        const ez = document.getElementById(`sm-appearance-variants-${category.theme_id}`)!;
        if (ez) {
          ez.style.display = "block";
          if (category.variant_theme_id) {
            (<HTMLSelectElement>document.getElementById(`sm-appearance-variants-select-${category.theme_id}`)!).value = category.variant_theme_id.toString();
          } else {
            (<HTMLSelectElement>document.getElementById(`sm-appearance-variants-select-${category.theme_id}`)!).value = category.theme_id.toString();
          }
        } else {
          category.variant_theme_id = null;
        }
      } else {
        document.getElementById("sm-appearance-noind")!.style.display = "block";
        if (category.theme_default == true) {
          (<HTMLInputElement>document.getElementById("sm-appearance-noin")!).checked = true;
          document.getElementById("sm-appearance-theme")!.style.display = "none";
        } else {
          (<HTMLInputElement>document.getElementById("sm-appearance-noin")!).checked = false;
          document.getElementById("sm-appearance-theme")!.style.display = "block";
          const ez = document.getElementById(`sm-appearance-variants-${category.theme_id}`)!;
          if (ez) {
            ez.style.display = "block";
            if (category.variant_theme_id) {
              (<HTMLSelectElement>document.getElementById(`sm-appearance-variants-select-${category.theme_id}`)!).value = category.variant_theme_id.toString();
            } else {
              (<HTMLSelectElement>document.getElementById(`sm-appearance-variants-select-${category.theme_id}`)!).value = category.theme_id.toString();
            }
          } else {
            category.variant_theme_id = null;
          }
        }
      }
      // if(category['theme_external_url']){
      (<HTMLInputElement>document.getElementById('sm-appearance-external-url')!).value = category.theme_external_url;
      // }

      (<HTMLSelectElement>document.getElementById("sm-appearance-theme-id")!).value = category.theme_id.toString();
      ManageSiteAppearanceModule.utils.updateThemePreview();
    },
    indClick: function (_event?: Event | null): void {
      const categoryId = parseInt((<HTMLSelectElement>document.getElementById("sm-appearance-cats")!).value);
      const category = Wikijump.modules.ManageSiteModule.utils.getCategoryById(categoryId);

      if ((<HTMLInputElement>document.getElementById("sm-appearance-noin")!).checked == true) {
        document.getElementById("sm-appearance-theme")!.style.display = "none";
        category.theme_default = true;
      } else {
        document.getElementById("sm-appearance-theme")!.style.display = "";
        category.theme_default = false;

        const ez = document.getElementById(`sm-appearance-variants-${category.theme_id}`)!;
        if (ez) {
          ez.style.display = "block";
          if (category.variant_theme_id) {
            (<HTMLSelectElement>document.getElementById(`sm-appearance-variants-select-${category.theme_id}`)!).value = category.variant_theme_id.toString();
          } else {
            (<HTMLSelectElement>document.getElementById(`sm-appearance-variants-select-${category.theme_id}`)!).value = category.theme_id.toString();
          }
        }
      }
      ManageSiteAppearanceModule.utils.updateThemePreview();
    },
    themeChange: function (_event?: Event | null): void {
      // save changes to the array
      const categoryId = parseInt((<HTMLSelectElement>document.getElementById("sm-appearance-cats")!).value);
      const category = Wikijump.modules.ManageSiteModule.utils.getCategoryById(categoryId);
      // @ts-expect-error What is this?
      if (this.tagName.toLowerCase() == 'select') {
        // @ts-expect-error What is this?
        category.theme_id = this.value;
      }
      ManageSiteAppearanceModule.utils.hideVariants();
      const ez = document.getElementById(`sm-appearance-variants-${(<HTMLSelectElement>document.getElementById("sm-appearance-theme-id")!).value}`);
      category.variant_theme_id = null;

      if (ez) {
        // alert('variant');
        ez.style.display = "block";
        if (category.variant_theme_id) {
          // XXX This will never happen
          (<HTMLSelectElement>document.getElementById(`sm-appearance-variants-select-${category.theme_id}`)!).value = category.variant_theme_id;
        } else if (document.getElementById(`sm-appearance-variants-select-${category.theme_id}`)!) {
          (<HTMLSelectElement>document.getElementById(`sm-appearance-variants-select-${category.theme_id}`)!).value = category.theme_id.toString();
        }
      } else {
        category.variant_theme_id = null;
      }

      /* Handle external themes. */
      const exurl = (<HTMLInputElement>document.getElementById('sm-appearance-external-url')!).value;
      // if(exurl != '' && exurl.match('^https?://')){
      category.theme_external_url = exurl;
      // }
      ManageSiteAppearanceModule.utils.updateThemePreview();
    },
    variantChange: function (_event?: Event | null): void {
      const categoryId = parseInt((<HTMLSelectElement>document.getElementById("sm-appearance-cats")!).value);
      const category = Wikijump.modules.ManageSiteModule.utils.getCategoryById(categoryId);
      const themeId = (<HTMLSelectElement>document.getElementById("sm-appearance-theme-id")!).value;
      const variantThemeId = (<HTMLSelectElement>document.getElementById(`sm-appearance-variants-select-${themeId}`)!).value;
      category.variant_theme_id = parseInt(variantThemeId);
    },
    cancel: function (_event?: Event | null): void {
      Wikijump.modules.ManageSiteModule.utils.loadModule('sm-welcome');
    },
    save: function (_event?: Event | null): void {
      // ok, do it the easy way: serialize categories using the JSON method
      const categories = Wikijump.modules.ManageSiteModule.vars.categories;
      const serialized = JSON.stringify(categories);
      // alert(serialized);
      const params: RequestModuleParameters = {
        categories: serialized,
        action: "ManageSiteAction",
        event: "saveAppearance"
      };
      const w = new OZONE.dialogs.WaitBox();
      w.content = "Saving changes...";
      w.show();
      OZONE.ajax.requestModule("Empty", params, ManageSiteAppearanceModule.callbacks.save);
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
    updateThemePreview: function (): void {
      const categoryId = parseInt((<HTMLSelectElement>document.getElementById("sm-appearance-cats")!).value);
      const category = Wikijump.modules.ManageSiteModule.utils.getCategoryById(categoryId);
      let themeId;
      // get current theme_id
      if ((<HTMLInputElement>document.getElementById("sm-appearance-noin")!).checked == true && category.name != "_default") {
        // get theme_id for the category _default
        const defCategory = Wikijump.modules.ManageSiteModule.utils.getCategoryByName("_default");
        themeId = defCategory.theme_id;
      } else {
        themeId = (<HTMLSelectElement>document.getElementById("sm-appearance-theme-id")!).value;
      }

      // hide all previews first
      const prs = <HTMLCollectionOf<HTMLElement>>document.getElementById("sm-appearance-theme-preview")!.children;
      for (let i = 0; i < prs.length; i++) {
        if (prs[i].tagName == 'div') {
          prs[i].style.display = "none";
        }
      }
      const previewDiv = document.getElementById(`sm-theme-preview-${themeId}`)!;
      if (previewDiv) {
        previewDiv.style.display = "block";
        document.getElementById("sm-appearance-theme-preview")!.style.display = "block";
      } else {
        document.getElementById("sm-appearance-theme-preview")!.style.display = "none";
      }
    },
    hideVariants: function (): void {
      const divs = document.getElementById("theme-variants-container")!.getElementsByTagName("div");
      for (let i = 0; i < divs.length; i++) {
        divs[i].style.display = "none";
      }
    }
  },
  init: function (): void {
    YAHOO.util.Event.addListener("sm-appearance-cats", "change", ManageSiteAppearanceModule.listeners.categoryChange);
    YAHOO.util.Event.addListener("sm-appearance-theme-id", "change", ManageSiteAppearanceModule.listeners.themeChange);
    YAHOO.util.Event.addListener("sm-appearance-external-url", "change", ManageSiteAppearanceModule.listeners.themeChange);
    YAHOO.util.Event.addListener("sm-appearance-noind", "click", ManageSiteAppearanceModule.listeners.indClick);

    YAHOO.util.Event.addListener("sm-appearance-cancel", "click", ManageSiteAppearanceModule.listeners.cancel);
    YAHOO.util.Event.addListener("sm-appearance-save", "click", ManageSiteAppearanceModule.listeners.save);

    // init categories info
    ManageSiteAppearanceModule.listeners.categoryChange(null);
  }
};

ManageSiteAppearanceModule.init();
