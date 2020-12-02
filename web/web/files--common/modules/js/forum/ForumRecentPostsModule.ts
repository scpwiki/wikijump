import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

export const ForumRecentPostsModule = {
  listeners: {
    updateList: function (pageNo?: number): void {
      const params: RequestModuleParameters = {
        page: pageNo ?? 1,
        categoryId: (<HTMLSelectElement>document.getElementById("recent-posts-category")!).value,
      };

      // Wikijump.modules.PageHistoryModule.vars.params = params; // for pagination

      OZONE.ajax.requestModule("forum/ForumRecentPostsListModule", params, ForumRecentPostsModule.callbacks.updateList);
    }
  },
  callbacks: {
    updateList: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }

      document.getElementById("forum-recent-posts-list")!.innerHTML = response.body;
      OZONE.utils.formatDates("forum-recent-posts-list");
      // OZONE.dialog.hovertip.makeTip(document.getElementById("forum-recent-posts-list")!.getElementsByTagName('span')
    }
  }
};
