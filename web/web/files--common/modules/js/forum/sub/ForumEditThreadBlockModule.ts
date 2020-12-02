import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

export const ForumEditThreadBlockModule = {
  listeners: {
    save: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        threadId: Wikijump.vars.forumThreadId,
        action: 'ForumAction',
        event: 'saveBlock'
      };
      if ((<HTMLInputElement>document.getElementById("thread-block-checkbox")!).checked) {
        params.block = true;
      }
      OZONE.ajax.requestModule(null, params, ForumEditThreadBlockModule.callbacks.save);
      const w = new OZONE.dialogs.WaitBox();
      w.content = "Saving changes...";
      w.show();
    }
  },
  callbacks: {
    save: function (response: YahooResponse): void {
      if (response.status != 'ok') {
        const w = new OZONE.dialogs.ErrorDialog();
        w.content = response.message;
        w.show();
        return;
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
