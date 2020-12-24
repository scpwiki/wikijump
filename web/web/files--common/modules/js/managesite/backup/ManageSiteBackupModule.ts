import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

export const ManageSiteBackupModule = {
  listeners: {
    requestBackup: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        ...OZONE.utils.formToArray("backup-form"),
        action: "ManageSiteBackupAction",
        event: "requestBackup",
      };
      OZONE.ajax.requestModule(null, params, ManageSiteBackupModule.callbacks.requestBackup);
    },
    deleteBackup: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        ...OZONE.utils.formToArray("backup-form"),
        action: "ManageSiteBackupAction",
        event: "deleteBackup",
      };
      OZONE.ajax.requestModule(null, params, ManageSiteBackupModule.callbacks.deleteBackup);
    }
  },
  callbacks: {
    requestBackup: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }

      // ok, reload the module now.
      Wikijump.modules.ManageSiteModule.utils.loadModule('sm-backup');
      OZONE.visuals.scrollTo('header');
    },
    deleteBackup: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }

      // ok, reload the module now.
      Wikijump.modules.ManageSiteModule.utils.loadModule('sm-backup');
      OZONE.visuals.scrollTo('header');
    }
  }
};
