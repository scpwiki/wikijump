import { compress } from "compress-tag";

import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

export const ForumEditPostFormModule = {
  listeners: {
    preview: function (_event?: Event | null): void {
      const params: RequestModuleParameters = OZONE.utils.formToArray("edit-post-form");
      OZONE.ajax.requestModule("forum/ForumPreviewPostModule", params, ForumEditPostFormModule.callbacks.preview);
    },
    cancel: function (_event?: Event | null): void {
      // remove form
      const formDiv = document.getElementById('edit-post-form-container')!;
      formDiv.parentNode!.removeChild(formDiv);

      Wikijump.Editor.shutDown();
    },
    closePreview: function (_event?: Event | null): void {
      document.getElementById("edit-post-preview-div")!.style.display = "none";
    },
    save: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        ...OZONE.utils.formToArray("edit-post-form"),
        action: "ForumAction",
        event: "saveEditPost"
      };
      const w = new OZONE.dialogs.WaitBox();
      w.content = "Saving changes...";
      w.show();
      OZONE.ajax.requestModule(null, params, ForumEditPostFormModule.callbacks.save);
    }
  },
  callbacks: {
    preview: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }

      const previewContainer = document.getElementById("edit-post-preview-div")!;
      previewContainer.getElementsByTagName('div')[0].innerHTML = response.body;
      previewContainer.style.visibility = "hidden";
      previewContainer.style.display = "block";

      // a trick. scroll first FAST and...
      previewContainer.style.visibility = "visible";
      OZONE.visuals.scrollTo("edit-post-preview-div");
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
      w.content = "Your changes have been saved.";
      w.show();
      const hash = `post-${response.postId}`;
      setTimeout(() => {
        window.location.hash = hash;
        window.location.reload();
      }, 1000);
    }
  }
};
