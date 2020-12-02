import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

export const ForumThreadMoveModule = {
  listeners: {
    move: function (_event?: Event | null): void {
      const categoryId = (<HTMLSelectElement>document.getElementById("move-thread-category")!).value;
      const params: RequestModuleParameters = {
        categoryId: categoryId,
        threadId: Wikijump.vars.forumThreadId,
        action: 'ForumAction',
        event: 'moveThread'
      };
      OZONE.ajax.requestModule(null, params, ForumThreadMoveModule.callbacks.move);
      const w = new OZONE.dialogs.WaitBox();
      w.content = "Moving thread...";
      w.show();
    }
  },
  callbacks: {
    move: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const w = new OZONE.dialogs.SuccessBox();
      w.content = "Thread has been moved.";
      w.show();
      setTimeout(() => window.location.reload(), 1000);
    }
  }
};
