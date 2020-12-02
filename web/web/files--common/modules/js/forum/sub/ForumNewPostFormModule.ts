import { compress } from "compress-tag";

import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
import { Wikirequest } from "wikirequest";
declare const WIKIREQUEST: Wikirequest;
declare type YahooResponse = any;

export const ForumNewPostFormModule = {
  listeners: {
    preview: function (_event?: Event | null): void {
      const params: RequestModuleParameters = OZONE.utils.formToArray("new-post-form");
      OZONE.ajax.requestModule("forum/ForumPreviewPostModule", params, ForumNewPostFormModule.callbacks.preview);
    },
    cancel: function (_event?: Event | null): void {
      // remove form
      const formDiv = document.getElementById('new-post-form-container')!;
      formDiv.parentNode!.removeChild(formDiv);
      document.getElementById("new-post-button")!.style.display = "";

      Wikijump.Editor.shutDown();
    },
    closePreview: function (_event?: Event | null): void {
      document.getElementById("new-post-preview-div")!.style.display = "none";
    },
    save: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        ...OZONE.utils.formToArray("new-post-form"),
        action: "ForumAction",
        event: "savePost"
      };
      const w = new OZONE.dialogs.WaitBox();
      w.content = "Posting now...";
      w.show();

      OZONE.ajax.requestModule(null, params, ForumNewPostFormModule.callbacks.save);
    }
  },
  callbacks: {
    preview: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }

      const previewContainer = document.getElementById("new-post-preview-div")!;

      const ctmp = previewContainer.getElementsByTagName('div');
      ctmp[0].innerHTML = response.body;
      previewContainer.style.visibility = "hidden";
      previewContainer.style.display = "block";

      // a trick. scroll first FAST and...
      previewContainer.style.visibility = "visible";
      OZONE.visuals.scrollTo("new-post-preview-div");
    },
    save: function (response: YahooResponse): void {
      if (response.status == "form_errors") {
        const errors = response.formErrors;
        const inner = compress`
          <p>
            The data you have submitted contains the following errors:
          </p>
          <ul>
            <li>
              ${errors.join("</li><li>")}
            </li>
          </ul>
        `;
        const w = new OZONE.dialogs.ErrorDialog();
        w.content = inner;
        w.show();
        return;
      }
      if (!Wikijump.utils.handleError(response)) { return; }

      const w = new OZONE.dialogs.SuccessBox();
      w.content = "Your post has been saved.";
      w.show();
      const hash = `post-${response.postId}`;
      // new uri:
      let uri = window.location.href.replace(/#.*$/, '');
      uri = uri.replace(/\/$/, '');
      if (!WIKIREQUEST.info.requestPageName.match(/^forum:thread$/)) {
        if (!uri.match(/comments\/show/)) {
          uri += '/comments/show';
          uri += `#post-${response.postId}`;
          setTimeout(() => window.location.href = uri, 1000);
        } else {
          setTimeout(() => {
            window.location.hash = hash;
            window.location.reload();
          }, 1000);
        }
      } else {
        setTimeout(() => {
          window.location.hash = hash;
          window.location.reload();
        }, 1000);
      }
    }
  }
};
