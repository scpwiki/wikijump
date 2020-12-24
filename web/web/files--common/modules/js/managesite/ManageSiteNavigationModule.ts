import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
import { Category } from "./ManageSiteModule";
import { Wikirequest } from "wikirequest";
declare const WIKIREQUEST: Wikirequest;
declare const YAHOO: any;
declare type YahooResponse = any;

export const ManageSiteNavigationModule = {
  vars: {
    currentCategory: null as null | Category
  },
  listeners: {
    categoryChange: function (_event?: Event | null): void {
      // update nav info
      const categoryId = parseInt((<HTMLSelectElement>document.getElementById("sm-nav-cats")!).value);
      const category = Wikijump.modules.ManageSiteModule.utils.getCategoryById(categoryId);
      ManageSiteNavigationModule.vars.currentCategory = category;
      // check if has a individual nav
      if (category.name == "_default") {
        document.getElementById("sm-nav-noind")!.style.display = "none";
        document.getElementById("sm-nav-list")!.style.display = "";
      } else {
        document.getElementById("sm-nav-noind")!.style.display = "block";
        if (category.nav_default) {
          (<HTMLInputElement>document.getElementById("sm-nav-noin")!).checked = true;
          document.getElementById("sm-nav-list")!.style.display = "none";
        } else {
          (<HTMLInputElement>document.getElementById("sm-nav-noin")!).checked = false;
          document.getElementById("sm-nav-list")!.style.display = "";
        }
      }

      ManageSiteNavigationModule.utils.updateNavigationPreview();
    },
    indClick: function (_event?: Event | null): void {
      const categoryId = parseInt((<HTMLSelectElement>document.getElementById("sm-nav-cats")!).value);
      const category = Wikijump.modules.ManageSiteModule.utils.getCategoryById(categoryId);

      if ((<HTMLInputElement>document.getElementById("sm-nav-noin")!).checked == true) {
        document.getElementById("sm-nav-list")!.style.display = "none";
        category.nav_default = true;
      } else {
        document.getElementById("sm-nav-list")!.style.display = "";
        category.nav_default = false;
      }
      ManageSiteNavigationModule.utils.updateNavigationPreview();
    },
    navChange: function (_event?: Event | null): void {
      // save changes to the array
      const category = ManageSiteNavigationModule.vars.currentCategory!;
      const topBar = (<HTMLInputElement>document.getElementById("sm-nav-top-bar")!).value;
      const sideBar = (<HTMLInputElement>document.getElementById("sm-nav-side-bar")!).value;
      category.top_bar_page_name = topBar;
      category.side_bar_page_name = sideBar;
      ManageSiteNavigationModule.utils.updateNavigationPreview();
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
        event: "saveNavigation"
      };
      OZONE.ajax.requestModule("Empty", params, ManageSiteNavigationModule.callbacks.save);
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
    updateNavigationPreview: function (): void {
      // apart from just updating the preview also show/hide extra textarea
      // for custom navs
      const category = ManageSiteNavigationModule.vars.currentCategory!;
      (<HTMLInputElement>document.getElementById("sm-nav-top-bar")!).value = category.top_bar_page_name;
      (<HTMLInputElement>document.getElementById("sm-nav-side-bar")!).value = category.side_bar_page_name;
    }
  },
  init: function (): void {
    YAHOO.util.Event.addListener("sm-nav-cats", "change", ManageSiteNavigationModule.listeners.categoryChange);

    YAHOO.util.Event.addListener("sm-nav-noind", "click", ManageSiteNavigationModule.listeners.indClick);

    const ids = ["sm-nav-top-bar", "sm-nav-side-bar"];
    YAHOO.util.Event.addListener(ids, 'keyup', ManageSiteNavigationModule.listeners.navChange);

    YAHOO.util.Event.addListener("sm-nav-cancel", "click", ManageSiteNavigationModule.listeners.cancel);
    YAHOO.util.Event.addListener("sm-nav-save", "click", ManageSiteNavigationModule.listeners.save);
    // init categories info
    ManageSiteNavigationModule.listeners.categoryChange(null);

    // attach the autocomplete thing
    const myDataSource = new YAHOO.widget.DS_XHR("/quickmodule.php", ['pages', 'unix_name', 'title']);
    myDataSource.scriptQueryParam = "q";
    myDataSource.scriptQueryAppend = `s=${WIKIREQUEST.info.siteId}&module=PageLookupQModule`;

    const myAutoComp = new YAHOO.widget.AutoComplete("sm-nav-top-bar", "sm-nav-top-bar-list", myDataSource);
    // @ts-expect-error Autocomp
    myAutoComp2.formatResult = function (aResultItem, _sQuery): string {
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
    myAutoComp.useIFrame = true;

    const myAutoComp2 = new YAHOO.widget.AutoComplete("sm-nav-side-bar", "sm-nav-side-bar-list", myDataSource);
    // @ts-expect-error Autocomp
    myAutoComp2.formatResult = function (aResultItem, _sQuery): string {
      const title = aResultItem[1];
      const unixName = aResultItem[0];
      if (unixName != null) {
        return `<div style="font-size: 100%">${unixName}</div><div style="font-size: 80%;">(${title})</div>`;
      } else {
        return "";
      }
    };

    myAutoComp.autoHighlight = false;
    myAutoComp2.minQueryLength = 2;
    myAutoComp2.queryDelay = 0.5;
  }
};

ManageSiteNavigationModule.init();
