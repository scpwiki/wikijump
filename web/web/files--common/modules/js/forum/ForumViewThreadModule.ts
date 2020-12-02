import { compress } from "compress-tag";

import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
import { Wikirequest } from "wikirequest";
declare const WIKIREQUEST: Wikirequest;
declare const YAHOO: any;
declare type YahooResponse = any;
declare const fx: any;

export const ForumViewThreadModule = {
  vars: {
  },
  listeners: {
    togglePostFold: function (_event: Event | null, postId: number): void {
      const foldedPost = document.getElementById(`post-${postId}`)!;

      const ofx = new fx.Opacity(foldedPost, {
        duration: 100,
        onComplete: function (): void {
          if (foldedPost.className.indexOf(' folded') >= 0) {
            foldedPost.className = foldedPost.className.replace(/ folded/, '');
          } else {
            foldedPost.className += ' folded';
          }

          const ofx = new fx.Opacity(foldedPost, { duration: 100 });
          ofx.setOpacity(0);
          ofx.custom(0, 1);
        }
      });
      ofx.custom(1, 0);
    },
    togglePostOptions: function (_event: Event | null, postId: number): void {
      const oDiv = document.getElementById(`post-options-${postId}`)!;
      if (oDiv.style.display != "block") {
        let inner = document.getElementById("post-options-template")!.innerHTML;
        // TODO This is a stupid system
        inner = inner.replace(/%POST_ID%/g, `${postId}`);
        oDiv.innerHTML = inner;

        // modify permalink...
        const els = oDiv.getElementsByTagName('a');
        for (let i = 0; i < els.length; i++) {
          if (els[i].innerHTML == 'permalink') {
            els[i].href = document.getElementById("post-options-permalink-template")!.innerHTML + postId;
          }
        }
        const ofx = new fx.Opacity(oDiv.id, { duration: 200 });
        ofx.setOpacity(0);
        oDiv.style.display = "block";
        ofx.custom(0, 1);
      } else {
        const ofx = new fx.Opacity(oDiv.id, { duration: 200 });
        ofx.custom(1, 0);
        setTimeout(() => document.getElementById(`post-options-${postId}`)!.style.display="none", 300);
      }
    },
    toggleThreadOptions: function (event: Event): void {
      const el = document.getElementById("thread-options-2")!;
      const ofx = new fx.Opacity(el, { duration: 200 });
      const t = YAHOO.util.Event.getTarget(event);
      if (el.style.display == 'none') {
        ofx.setOpacity(0);
        el.style.display = "block";
        ofx.custom(0, 1);
        t.innerHTML = "- less options";
      } else {
        ofx.custom(1, 0);
        t.innerHTML = "+ more options";
        setTimeout(() => document.getElementById("thread-options-2")!.style.display="none", 200);
      }
    },
    showPermalink: function (_event: Event | null, postId: string): void {
      const w = new OZONE.dialogs.InfoDialog();
      w.style = { width: "60em" };
      // w.content='<h1>Permanent link</h1><params>Permanent link for this post is:</params>' +

      let uri = window.location.href.replace(/#.*$/, '').replace(/\/$/, '');
      if (!WIKIREQUEST.info.requestPageName.match(/^forum:thread$/)) {
        if (!uri.match(/comments\/show/)) {
          uri += '/comments/show';
        }
      }
      uri += "#post-" + postId;

      w.content = compress`
        <h1>Permanent link</h1>
        <p>Permanent link for this post is:</p>
        <p><strong>${uri}</strong></p>
      `;

      w.show();
    },
    newPost: function (_event: Event | null, postId: number | null): void {
      if (Wikijump.Editor.editElementId) {
        const w = new OZONE.dialogs.ErrorDialog();
        w.content = compress`
          <p>
            You have an active editor somewhere already and it is not possible
            to edit multiple elements at once.
          </p>
          <p>
            (
            <a href="javascript:;"
               onclick="OZONE.visuals.scrollTo('${Wikijump.Editor.editElementId}');
                        OZONE.dialog.cleanAll()">
              scroll to active editor
            </a>
            )
          </p>
        `;
        w.show();
        return;
      }
      // postId is an optional postId to reply to.
      const params: RequestModuleParameters = {
        postId: postId,
        threadId: Wikijump.vars.forumThreadId
      };
      OZONE.ajax.requestModule('forum/sub/ForumNewPostFormModule', params, ForumViewThreadModule.callbacks.newPost);
    },
    editPost: function (_event: Event | null, postId: string): void {
      if (Wikijump.Editor.editElementId) {
        const w = new OZONE.dialogs.ErrorDialog();
        w.content = compress`
          <p>
            You have an active editor somewhere already and it is not possible
            to edit multiple elements at once.
          </p>
          <p>
            (
            <a href="javascript:;"
               onclick="OZONE.visuals.scrollTo('${Wikijump.Editor.editElementId}');
                        OZONE.dialog.cleanAll()">
              scroll to active editor
            </a>
            )
          </p>
        `;
        w.show();
        return;
      }
      const params: RequestModuleParameters = {
        postId: postId,
        threadId: Wikijump.vars.forumThreadId
      };
      OZONE.ajax.requestModule('forum/sub/ForumEditPostFormModule', params, ForumViewThreadModule.callbacks.editPost);
    },
    deletePost: function (_event: Event | null, postId: number): void {
      OZONE.ajax.requestModule("forum/sub/ForumDeletePostModule", { postId: postId }, ForumViewThreadModule.callbacks.deletePost);
    },
    foldAll: function (_event?: Event | null): void {
      const posts = YAHOO.util.Dom.getElementsByClassName("post", 'div');
      for (let i = 0; i < posts.length; i++) {
        YAHOO.util.Dom.addClass(posts[i], "folded");
      }
    },
    unfoldAll: function (_event?: Event | null): void {
      const posts = YAHOO.util.Dom.getElementsByClassName("post", 'div');
      for (let i = 0; i < posts.length; i++) {
        YAHOO.util.Dom.removeClass(posts[i], "folded");
      }
    },
    showHistory: function (_event: Event | null, postId: number): void {
      const params: RequestModuleParameters = {
        postId: postId
      };
      OZONE.ajax.requestModule("forum/sub/ForumPostRevisionsModule", params, ForumViewThreadModule.callbacks.showHistory);
    },
    hideHistory: function (_event: Event | null, postId: number): void {
      const postDiv = document.getElementById("post-" + postId)!;
      const revDiv = YAHOO.util.Dom.getElementsByClassName('revisions', 'div', postDiv)[0];
      const chDiv = YAHOO.util.Dom.getElementsByClassName('changes', 'div', postDiv)[0];
      chDiv.style.display = "block";
      revDiv.style.display = "none";
    },
    showRevision: function (event: Event, revisionId: number): void {
      const params: RequestModuleParameters = {
        revisionId: revisionId
      };
      OZONE.ajax.requestModule("forum/sub/ForumPostRevisionModule", params, ForumViewThreadModule.callbacks.showRevision);
      // clear active

      let t = YAHOO.util.Event.getTarget(event);

      let t2 = t.parentNode;
      while (!t2.tagName || t2.tagName.toLowerCase() != 'table') {
        t2 = t2.parentNode;
      }
      const tact = YAHOO.util.Dom.getElementsByClassName('active', 'tr', t2)[0];
      YAHOO.util.Dom.removeClass(tact, 'active');

      while (!t.tagName || t.tagName.toLowerCase() != 'tr') {
        t = t.parentNode;
      }
      YAHOO.util.Dom.addClass(t, "active");
    },
    editThreadMeta: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        threadId: Wikijump.vars.forumThreadId
      };
      OZONE.ajax.requestModule("forum/sub/ForumEditThreadMetaModule", params, ForumViewThreadModule.callbacks.editThreadMeta);
    },
    editThreadStickiness: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        threadId: Wikijump.vars.forumThreadId
      };
      OZONE.ajax.requestModule("forum/sub/ForumEditThreadStickinessModule", params, ForumViewThreadModule.callbacks.editThreadStickiness);
    },
    editThreadBlock: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        threadId: Wikijump.vars.forumThreadId
      };
      OZONE.ajax.requestModule("forum/sub/ForumEditThreadBlockModule", params, ForumViewThreadModule.callbacks.editThreadBlock);
    },
    moveThread: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        threadId: Wikijump.vars.forumThreadId
      };
      OZONE.ajax.requestModule("forum/sub/ForumThreadMoveModule", params, ForumViewThreadModule.callbacks.moveThread);
    },
    watchThread: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        threadId: Wikijump.vars.forumThreadId,
        action: "WatchAction",
        event: "watchThread"
      };

      OZONE.ajax.requestModule(null, params, ForumViewThreadModule.callbacks.watchThread);
    }
  },
  callbacks: {
    newPost: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      // proceed
      const parentId = response.parentId;

      const formDiv = document.createElement('div');
      formDiv.id = "new-post-form-container";
      formDiv.innerHTML = response.body;
      // find the location for the form-div and insert....
      if (parentId == null) {
        // append at the end of "forum-posts-container"
        const forumPostsContainer = document.getElementById("thread-container")!;
        forumPostsContainer.appendChild(formDiv);
        // hide "new post" button
        document.getElementById("new-post-button")!.style.display = "none";
      } else {
        const postContainer = document.getElementById("fpc-" + parentId)!;
        const post = document.getElementById("post-" + parentId)!;
        if (response.parentChanged == true) {
          postContainer.appendChild(formDiv);
        } else {
          OZONE.dom.insertAfter(postContainer, formDiv, post);
        }
      }

      // init editor
      Wikijump.Editor.init("np-text", "np-editor-panel");

      setTimeout(() => OZONE.visuals.scrollTo("new-post-form-container"), 300);
    },
    editPost: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const formDiv = document.createElement('div');
      formDiv.id = "edit-post-form-container";
      formDiv.innerHTML = response.body;
      // now where to put it....
      const postContainer = document.getElementById("fpc-" + response.postId)!;
      const post = document.getElementById("post-" + response.postId)!;
      OZONE.dom.insertAfter(postContainer, formDiv, post);

      Wikijump.Editor.init("np-text", "np-editor-panel");
      setTimeout(() => OZONE.visuals.scrollTo("edit-post-form-container"), 300);
    },
    showHistory: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }

      const postDiv = document.getElementById("post-" + response.postId)!;
      const revDiv = YAHOO.util.Dom.getElementsByClassName('revisions', 'div', postDiv)[0];
      const chDiv = YAHOO.util.Dom.getElementsByClassName('changes', 'div', postDiv)[0];
      chDiv.style.display = "none";

      revDiv.innerHTML = response.body;
      revDiv.style.display = "block";
      OZONE.utils.formatDates(revDiv);
    },
    showRevision: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }

      const postId = response.postId;
      document.getElementById("post-content-" + postId)!.innerHTML = response.content;
      document.getElementById("post-title-" + postId)!.innerHTML = response.title;
    },
    deletePost: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const po = document.getElementById("post-" + response.postId)!;
      const co = document.getElementById("fpc-" + response.postId)!;
      YAHOO.util.Dom.addClass(co, "fordelete");
      const id = "delete-post-" + response.postId;
      if (document.getElementById(id)) {
        document.getElementById(id)!.parentNode!.removeChild(document.getElementById(id)!);
      }
      po.innerHTML += response.body;
      OZONE.visuals.scrollTo(id);
    },
    editThreadMeta: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const el = document.getElementById("thread-action-area")!;
      el.style.display = "block";
      el.innerHTML = response.body;
      new OZONE.forms.lengthLimiter("thread-description", "desc-charleft", 1000);
    },
    editThreadStickiness: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const el = document.getElementById("thread-action-area")!;
      el.style.display = "block";
      el.innerHTML = response.body;
    },
    editThreadBlock: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const el = document.getElementById("thread-action-area")!;
      el.style.display = "block";
      el.innerHTML = response.body;
    },
    moveThread: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const el = document.getElementById("thread-action-area")!;
      el.style.display = "block";
      el.innerHTML = response.body;
    },
    watchThread: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }

      const w = new OZONE.dialogs.SuccessBox();
      w.content = "Thead added to watched.";
      w.show();
    }

  }
};

// shortcut functions versions

// @ts-expect-error templates/macros/Forum.tpl
window.togglePostOptions = function (event: Event | null, postId: number): void {
  ForumViewThreadModule.listeners.togglePostOptions(event, postId);
};
// @ts-expect-error templates/macros/Forum.tpl
window.togglePostFold = function (event: Event | null, postId: number): void {
  ForumViewThreadModule.listeners.togglePostFold(event, postId);
};
// @ts-expect-error templates/macros/Forum.tpl
window.postReply = function (event: Event | null, postId: number): void {
  ForumViewThreadModule.listeners.newPost(event, postId);
};
