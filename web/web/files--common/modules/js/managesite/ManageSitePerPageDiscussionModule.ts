import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

export const ManageSitePerPageDiscussionModule = {
  listeners: {
    save: function (_event?: Event | null): void {
      ManageSitePerPageDiscussionModule.utils.updateFromForm();
      const categories = Wikijump.modules.ManageSiteModule.vars.categories;
      const serialized = JSON.stringify(categories);
      const params: RequestModuleParameters = {
        categories: serialized,
        action: "ManageSiteForumAction",
        event: "savePerPageDiscussion"
      };
      OZONE.ajax.requestModule("Empty", params, ManageSitePerPageDiscussionModule.callbacks.save);
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
        id = "cat234-" + categories[i].category_id;
        if ((<HTMLInputElement>document.getElementById(id + "-e")!).checked) {
          categories[i].per_page_discussion = true;
        } else if ((<HTMLInputElement>document.getElementById(id + "-d")!).checked) {
          categories[i].per_page_discussion = false;
        } else {
          categories[i].per_page_discussion = null;
        }
      }
    }

  }
};
