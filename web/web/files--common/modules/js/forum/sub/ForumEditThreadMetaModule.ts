import { compress } from "compress-tag";

import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

export const ForumEditThreadMetaModule = {
  listeners: {
    save: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        ...OZONE.utils.formToArray("thread-meta-form"),
        action: 'ForumAction',
        event: 'saveThreadMeta'
      };
      OZONE.ajax.requestModule(null, params, ForumEditThreadMetaModule.callbacks.save);
      const w = new OZONE.dialogs.WaitBox();
      w.content = "Saving changes...";
      w.show();
    }
  },
  callbacks: {
    save: function (response: YahooResponse): void {
      if (response.status == "form_errors") {
        OZONE.dialog.cleanAll();
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
        document.getElementById("thread-meta-errors")!.innerHTML = inner;
        document.getElementById("thread-meta-errors")!.style.display = "block";
        return;
      }
      if (!Wikijump.utils.handleError(response)) { return; }

      const w = new OZONE.dialogs.SuccessBox();
      w.content = "Your changes have been saved.";
      w.show();

      setTimeout(() => {
        window.location.hash = "";
        window.location.reload();
      }, 1000);
    }
  }
};
