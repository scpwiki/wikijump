import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

export const AWChangesModule = {
  listeners: {
    showWatchedPages: function (_event?: Event | null): void {
      OZONE.ajax.requestModule("/account/watch/AWPagesListModule", {}, AWChangesModule.callbacks.showWatchedPages);
    },
    hideWatchedPages: function (_event?: Event | null): void {
      document.getElementById("watched-pages-list")!.innerHTML = "";
      document.getElementById("watched-pages-list")!.style.display = "none";
      document.getElementById("show-watched-pages-button")!.style.display = "";
      document.getElementById("hide-watched-pages-button")!.style.display = "none";
    },
    removeWatchedPage: function (_event: Event | null, pageId: number): void {
      const params: RequestModuleParameters = {
        pageId: pageId,
        action: "WatchAction",
        event: "removeWatchedPage"
      };
      OZONE.ajax.requestModule(null, params, AWChangesModule.callbacks.removeWatchedPage);
    },
    updateList: function (_event?: Event | null, pageNo?: number): void {
      const params: RequestModuleParameters = {
        page: pageNo ?? 1
      };
      OZONE.ajax.requestModule("account/watch/AWChangesListModule", params, AWChangesModule.callbacks.updateList);
    }
  },
  callbacks: {
    showWatchedPages: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const el = document.getElementById("watched-pages-list")!;
      el.innerHTML = response.body;
      el.style.display = "block";
      document.getElementById("hide-watched-pages-button")!.style.display = "";
      document.getElementById("show-watched-pages-button")!.style.display = "none";
    },
    removeWatchedPage: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const w = new OZONE.dialogs.SuccessBox();
      w.content = "Page not being watched any more.";
      w.show();
      AWChangesModule.listeners.showWatchedPages();
      AWChangesModule.listeners.updateList();
    },
    updateList: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }

      document.getElementById("watched-changes-list")!.innerHTML = response.body;
      OZONE.utils.formatDates("watched-changes-list");
      OZONE.dialog.hovertip.makeTip(document.getElementById("watched-changes-list")!.getElementsByTagName('span'),
                                    { style: { width: 'auto' } });
    }
  },
  init: function (): void {
    OZONE.utils.formatDates("watched-changes-list");
    OZONE.dialog.hovertip.makeTip(document.getElementById("watched-changes-list")!.getElementsByTagName('span'),
                                  { style: { width: 'auto' } });
  }
};

AWChangesModule.init();
