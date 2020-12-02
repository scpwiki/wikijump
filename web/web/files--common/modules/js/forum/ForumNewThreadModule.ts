import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare const YAHOO: any;
declare type YahooResponse = any;

export const ForumNewThreadModule = {
  vars: {
    // Set by template
    cancelUrl: null as null | string
  },
  listeners: {
    cancel: function (_event?: Event | null): void {
      window.location.href = ForumNewThreadModule.vars.cancelUrl!;
    },
    preview: function (_event?: Event | null): void {
      const params: RequestModuleParameters = OZONE.utils.formToArray("new-thread-form");
      OZONE.ajax.requestModule("forum/ForumPreviewPostModule", params, ForumNewThreadModule.callbacks.preview);
    },
    post: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        ...OZONE.utils.formToArray("new-thread-form"),
        action: "ForumAction",
        event: "newThread"
      };
      const w = new OZONE.dialogs.WaitBox();
      w.content = "Creating new thread...";
      w.show();
      OZONE.ajax.requestModule("Empty", params, ForumNewThreadModule.callbacks.post);
    }
  },
  callbacks: {
    preview: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      document.getElementById("message-preview-wrapper")!.style.display = "block";
      OZONE.utils.setInnerHTMLContent("message-preview", response.body);
      OZONE.visuals.scrollTo("message-preview");
    },
    post: function (response: YahooResponse): void {
      if (response.status == "form_errors") {
        OZONE.dialog.cleanAll();
        let inner = "The data you have submitted contains following errors:" +
          "<ul>";

        const errors = response.formErrors;
        for (const i in errors) {
          inner += "<li>" + errors[i] + "</li>";
        }
        inner += "</ul>";
        document.getElementById("new-thread-error")!.innerHTML = inner;
        document.getElementById("new-thread-error")!.style.display = "block";
        return;
      }
      if (!Wikijump.utils.handleError(response)) { return; }
      const w = new OZONE.dialogs.SuccessBox();
      w.content = "Thread has been created.";
      w.show();

      const uri = `/forum/t-${response.threadId}/${response.threadUnixifiedTitle}`;
      setTimeout(() => window.location.href = uri, 1000);
    }
  },
  init: function (): void {
    YAHOO.util.Event.addListener("ntf-cancel", "click", ForumNewThreadModule.listeners.cancel);
    YAHOO.util.Event.addListener("ntf-preview", "click", ForumNewThreadModule.listeners.preview);
    YAHOO.util.Event.addListener("ntf-post", "click", ForumNewThreadModule.listeners.post);

    OZONE.dom.onDomReady(function (): void {
      Wikijump.Editor.init("post-edit", "post-edit-panel");
      new OZONE.forms.lengthLimiter("thread-description", "desc-charleft", 1000);
    }, "dummy-ondomready-block");
  }
};

ForumNewThreadModule.init();
