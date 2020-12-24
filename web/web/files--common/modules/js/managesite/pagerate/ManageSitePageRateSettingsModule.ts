import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

export const ManageSitePageRateSettingsModule = {
  listeners: {
    save: function (_event?: Event | null): void {
      ManageSitePageRateSettingsModule.utils.updateFromForm();
      const categories = Wikijump.modules.ManageSiteModule.vars.categories;
      const serialized = JSON.stringify(categories);
      const params: RequestModuleParameters = {
        categories: serialized,
        action: "ManageSiteAction",
        event: "savePageRateSettings"
      };

      OZONE.ajax.requestModule("Empty", params, ManageSitePageRateSettingsModule.callbacks.save);
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
  utils: {
    updateFromForm: function (): void {
      const categories = Wikijump.modules.ManageSiteModule.vars.categories!;
      let id;
      for (let i = 0; i < categories.length; i++) {
        // check for the value in the form
        id = "cat235-" + categories[i].category_id;
        let ps = '';
        if ((<HTMLSelectElement>document.getElementById(id + "-event")!).value == 'enabled') {
          ps += 'event';
        } else if ((<HTMLSelectElement>document.getElementById(id + "-event")!).value == 'disabled') {
          ps += 'd';
        }

        if ((<HTMLSelectElement>document.getElementById(id + "-w")!).value == 'response') {
          ps += 'response';
        } else if ((<HTMLSelectElement>document.getElementById(id + "-w")!).value == 'm') {
          ps += 'm';
        }

        if ((<HTMLSelectElement>document.getElementById(id + "-v")!).value == 'v') {
          ps += 'v';
        } else if ((<HTMLSelectElement>document.getElementById(id + "-v")!).value == 'a') {
          ps += 'a';
        }

        ps += (<HTMLSelectElement>document.getElementById(id + "-t")!).value;

        categories[i].rating = ps;
      }
    },
    updateVis: function (categoryId: number): void {
      const id = "cat235-" + categoryId;
      if ((<HTMLInputElement>document.getElementById(id + "-event")!).value == 'enabled') {
        document.getElementById(id + "-w")!.style.visibility = "visible";
        document.getElementById(id + "-v")!.style.visibility = "visible";
        document.getElementById(id + "-t")!.style.visibility = "visible";
      } else {
        document.getElementById(id + "-w")!.style.visibility = "hidden";
        document.getElementById(id + "-v")!.style.visibility = "hidden";
        document.getElementById(id + "-t")!.style.visibility = "hidden";
      }
    }
  },
  init: function (): void {
    const categories = Wikijump.modules.ManageSiteModule.vars.categories!;
    for (let i = 0; i < categories.length; i++) {
      ManageSitePageRateSettingsModule.utils.updateVis(categories[i].category_id);
    }
  }
};

ManageSitePageRateSettingsModule.init();
