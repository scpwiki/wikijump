import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

export const ForumEditThreadStickinessModule = {
  listeners: {
    save: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        threadId: Wikijump.vars.forumThreadId,
        action: 'ForumAction',
        event: 'saveSticky',
      };
      if ((<HTMLInputElement>document.getElementById("thread-sticky-checkbox")!).checked) {
        params.sticky = true;
      }
      OZONE.ajax.requestModule(null, params, ForumEditThreadStickinessModule.callbacks.save);
      const w = new OZONE.dialogs.WaitBox();
      w.content = "Saving changes...";
      w.show();
    }
  },
  callbacks: {
    save: function (response: YahooResponse): void {
      if (response.status != 'ok') {
        const w = new OZONE.dialogs.ErrorDialog();	w.content = response.message; w.show();	return;
      }
      const w = new OZONE.dialogs.SuccessBox();
      w.content = "Your changes have been saved.";
      w.show();

      setTimeout(() => {
        window.location.hash="";
        window.location.reload();
      }, 1000);
    }
  }
};
