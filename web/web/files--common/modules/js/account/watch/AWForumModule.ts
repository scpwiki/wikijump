import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

export const AWForumModule = {
  listeners: {
    showWatchedThreads: function (_event?: Event | null): void {
      OZONE.ajax.requestModule("/account/watch/AWThreadsListModule", {}, AWForumModule.callbacks.showWatchedThreads);
    },
    hideWatchedThreads: function (_event?: Event | null): void {
      document.getElementById("watched-threads-list")!.innerHTML = "";
      document.getElementById("watched-threads-list")!.style.display = "none";
      document.getElementById("show-watched-threads-button")!.style.display = "";
      document.getElementById("hide-watched-threads-button")!.style.display = "none";
    },
    removeWatchedThread: function (_event: Event | null, threadId: number): void {
      const params: RequestModuleParameters = {
        threadId: threadId,
        action: "WatchAction",
        event: "removeWatchedThread"
      };
      OZONE.ajax.requestModule(null, params, AWForumModule.callbacks.removeWatchedThread);
    },
    updateList: function (_event?: Event | null, pageNo?: number): void {
      const params: RequestModuleParameters = {
        page: pageNo ?? 1
      };
      OZONE.ajax.requestModule("account/watch/AWForumListModule", params, AWForumModule.callbacks.updateList);
    }
  },
  callbacks: {
    showWatchedThreads: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const el = document.getElementById("watched-threads-list")!;
      el.innerHTML = response.body;
      el.style.display = "block";
      document.getElementById("hide-watched-threads-button")!.style.display = "";
      document.getElementById("show-watched-threads-button")!.style.display = "none";
    },
    removeWatchedThread: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const w = new OZONE.dialogs.SuccessBox();
      w.content = "Thread not being watched any more.";
      w.show();
      AWForumModule.listeners.showWatchedThreads();
      AWForumModule.listeners.updateList();
    },
    updateList: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }

      document.getElementById("watched-forum-list")!.innerHTML = response.body;
      OZONE.utils.formatDates("watched-forum-list");
    }

  }
};
