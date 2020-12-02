import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
import { Wikirequest } from "wikirequest";
declare const WIKIREQUEST: Wikirequest;
declare type YahooResponse = any;

export const ForumCommentsModule = {
  listeners: {
    showComments: function (_event?: Event | null): void {
      // if "thread-container" is filled with data - just show it.
      // if not - make an ajax request for the content
      const tc = document.getElementById("thread-container")!;
      if (tc.innerHTML.match(/^[\s\n\response]*$/)) {
        tc.innerHTML = '<div class="wait-block">Loading comments...</div>';
        const params: RequestModuleParameters = {
          pageId: WIKIREQUEST.info.pageId
        };
        OZONE.ajax.requestModule("forum/ForumCommentsListModule", params, ForumCommentsModule.callbacks.showComments);
      } else {
        tc.style.display = "block";
        document.getElementById("comments-options-hidden")!.style.display = "none";
        document.getElementById("comments-options-shown")!.style.display = "block";
      }
    },
    hideComments: function (_event?: Event | null): void {
      const tc = document.getElementById("thread-container")!;
      tc.style.display = "none";
      document.getElementById("comments-options-hidden")!.style.display = "block";
      document.getElementById("comments-options-shown")!.style.display = "none";
    }
  },
  callbacks: {
    showComments: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }

      const tc = document.getElementById("thread-container")!;
      OZONE.utils.setInnerHTMLContent("thread-container", response.body);
      tc.style.display = "block";
      document.getElementById("comments-options-hidden")!.style.display = "none";
      document.getElementById("comments-options-shown")!.style.display = "block";

      Wikijump.vars.forumThreadId = response.threadId;
    }
  }
};
