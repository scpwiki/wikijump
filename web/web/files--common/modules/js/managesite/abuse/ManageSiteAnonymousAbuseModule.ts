import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

export const ManageSiteAnonymousAbuseModule = {
  vars: {
    currentIP: null as null | string
  },
  listeners: {
    clear: function (_event: Event | null, address: string, proxy?: 'proxy'): void {
      const params: RequestModuleParameters = {
        action: "ManageSiteAbuseAction",
        event: "clearAnonymousFlags",
        address: address
      };
      if (proxy == 'proxy') {
        params.proxy = "yes";
      }
      OZONE.ajax.requestModule(null, params, ManageSiteAnonymousAbuseModule.callbacks.clear);
    },
    blockIp: function (_event: Event | null, address: string): void {
      ManageSiteAnonymousAbuseModule.vars.currentIP = address;
      const w = new OZONE.dialogs.ConfirmationDialog();
      w.content = document.getElementById("ban-ip-dialog")!.innerHTML.replace(/%%IP%%/, address);
      w.buttons = ['cancel', 'yes, ban'];
      w.addButtonListener('cancel', w.close);
      w.addButtonListener('yes, ban', ManageSiteAnonymousAbuseModule.listeners.blockIp2);
      w.show();
    },
    blockIp2: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        ips: ManageSiteAnonymousAbuseModule.vars.currentIP,
        action: "ManageSiteBlockAction",
        event: "blockIp",
      };
      OZONE.ajax.requestModule(null, params, ManageSiteAnonymousAbuseModule.callbacks.blockIp);
    }
  },
  callbacks: {
    clear: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const w = new OZONE.dialogs.SuccessBox();
      w.content = "Flags cleared";
      w.show();
    },
    blockIp: function (response: YahooResponse): void {
      if (response.status !== 'ok') {
        const w = new OZONE.dialogs.ErrorDialog();
        w.content = response.message;
        if (response.errormess) {
          w.content += '<br/>' + response.errormess;
        }
        w.show();
        return;
      }

      const w = new OZONE.dialogs.SuccessBox();
      w.content = "The IP address added to the block list.";
      w.show();
    }
  }
};
