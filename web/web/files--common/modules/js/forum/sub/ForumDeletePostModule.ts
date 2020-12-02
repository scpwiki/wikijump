import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare const YAHOO: any;
declare type YahooResponse = any;

export const ForumDeletePostModule = {
  listeners: {
    cancel: function (_event: Event | null, postId: number): void {
      const co = document.getElementById("fpc-" + postId)!;
      YAHOO.util.Dom.removeClass(co, "fordelete");
      const id = "delete-post-" + postId;
      if (document.getElementById(id)) {
        document.getElementById(id)!.parentNode!.removeChild(document.getElementById(id)!);
      }
    },
    deletePost: function (_event: Event | null, postId: number): void {
      const params: RequestModuleParameters = {
        action: "ForumAction",
        event: "deletePost",
        postId: postId
      };
      OZONE.ajax.requestModule(null, params, ForumDeletePostModule.callbacks.deletePost);
      const w = new OZONE.dialogs.WaitBox();
      w.content = "Deleting post...";
      w.show();
    }
  },
  callbacks: {
    deletePost: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const w = new OZONE.dialogs.SuccessBox();
      w.content = "The post has been deleted.";
      w.show();
      setTimeout(() => window.location.reload(), 1000);
    }
  }
};
