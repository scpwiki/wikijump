import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

export const ManageSiteIpBlocksModule = {
  vars: {
    addFormInited: false,
    currentIp: null as null | string,
    dCurrentBlockId: null as null | number
  },
  listeners: {
    showAddForm: function (_event?: Event | null): void {
      if (!ManageSiteIpBlocksModule.vars.addFormInited) {
        new OZONE.forms.lengthLimiter("block-reason", "reason-char-left", 200);
        ManageSiteIpBlocksModule.vars.addFormInited = true;
      }
      document.getElementById("show-add-block-button")!.style.display = "none";
      document.getElementById("add-block-div")!.style.display = "block";
      OZONE.visuals.scrollTo("add-block-div");
    },
    cancelAdd: function (_event?: Event | null): void {
      // resets the forms?
      document.getElementById("show-add-block-button")!.style.display = "block";
      document.getElementById("add-block-div")!.style.display = "none";
      document.getElementById("ip-errors")!.style.display = "none";
    },
    blockIp: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        ips: (<HTMLTextAreaElement>document.getElementById("block-ips")!).value,
        reason: (<HTMLTextAreaElement>document.getElementById("block-reason")!).value,
        action: "ManageSiteBlockAction",
        event: "blockIp",
      };
      if (params.ips == '') {
        const w = new OZONE.dialogs.ErrorDialog();
        w.content = "IP address(es) field is blank.";
        w.show();
        return;
      }
      OZONE.ajax.requestModule(null, params, ManageSiteIpBlocksModule.callbacks.blockIp);
    },
    deleteBlock: function (_event: Event | null, blockId: number, ip: string): void {
      const w = new OZONE.dialogs.ConfirmationDialog();
      w.buttons = ['cancel', 'yes, delete block'];
      w.addButtonListener('cancel', w.close);
      w.addButtonListener('yes, delete block', ManageSiteIpBlocksModule.listeners.deleteBlock2, ip);
      w.content = "Are you sure you want to remove the block for the IP <strong>" + ip + "</strong>?";
      w.show();
      ManageSiteIpBlocksModule.vars.dCurrentBlockId = blockId;
    },
    deleteBlock2: function (_event?: Event | null): void {
      const blockId = ManageSiteIpBlocksModule.vars.dCurrentBlockId;
      const params: RequestModuleParameters = {
        blockId: blockId,
        action: "ManageSiteBlockAction",
        event: "deleteIpBlock"
      };
      OZONE.ajax.requestModule(null, params, ManageSiteIpBlocksModule.callbacks.deleteBlock);
    }
  },
  callbacks: {
    blockIp: function (response: YahooResponse): void {
      if (response.status == 'ip_errors') {
        document.getElementById("ip-errors")!.innerHTML = response.errormess;
        document.getElementById("ip-errors")!.style.display = "block";
        return;
      }
      if (!Wikijump.utils.handleError(response)) { return; }

      const w = new OZONE.dialogs.SuccessBox();
      w.content = "IP(s) added to the block list.";
      w.show();
      // refresh the screen too
      setTimeout(() => Wikijump.modules.ManageSiteModule.utils.loadModule("sm-ip-blocks"), 1500);
    },
    deleteBlock: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }

      const w = new OZONE.dialogs.SuccessBox();
      w.content = "IP block removed.";
      w.show();
      // refresh the screen too
      setTimeout(() => Wikijump.modules.ManageSiteModule.utils.loadModule("sm-ip-blocks"), 1500);
    }
  }
};
