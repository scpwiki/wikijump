import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

export const ManageSiteForumSettingsModule = {
  listeners: {
    activateForum: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        action: "ManageSiteForumAction",
        event: "activateForum"
      };

      OZONE.ajax.requestModule(null, params, ManageSiteForumSettingsModule.callbacks.activateForum);
      const w = new OZONE.dialogs.WaitBox();
      w.content = "Initializing forum...";
      w.show();
    },
    saveNesting: function (_event?: Event | null): void {
      const nest = (<HTMLSelectElement>document.getElementById("max-nest-level")!).value;
      const params: RequestModuleParameters = {
        action: "ManageSiteForumAction",
        event: "saveForumDefaultNesting",
        max_nest_level: nest
      };
      OZONE.ajax.requestModule("Empty", params, ManageSiteForumSettingsModule.callbacks.saveNesting);
      const w = new OZONE.dialogs.WaitBox();
      w.content = "Saving changes...";
      w.show();
    }
  },
  callbacks: {
    saveNesting: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const w = new OZONE.dialogs.SuccessBox();
      w.content = "Changes saved.";
      w.show();
    },
    activateForum: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const w = new OZONE.dialogs.SuccessBox();
      w.content = "Forum has been activated.";
      w.show();
      setTimeout(() => Wikijump.modules.ManageSiteModule.utils.loadModule('sm-forum-settings'), 1000);
    }
  }
};
