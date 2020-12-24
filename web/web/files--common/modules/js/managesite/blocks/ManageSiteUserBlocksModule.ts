import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare const YAHOO: any;
declare type YahooResponse = any;

export const ManageSiteUserBlocksModule = {
  vars: {
    addFormInited: false,
    currentUserId: null as null | number,
    dCurrentUserId: null as null | number
  },
  listeners: {
    showAddForm: function (_event?: Event | null): void {
      if (!ManageSiteUserBlocksModule.vars.addFormInited) {
        // init autocomplete now
        const dataSource = new YAHOO.widget.DS_XHR("/quickmodule.php", ['users', 'name', 'user_id']);
        dataSource.scriptQueryParam = "q";
        dataSource.scriptQueryAppend = "&module=UserLookupQModule";

        const autoComp = new YAHOO.widget.AutoComplete('user-lookup', 'user-lookup-list', dataSource);

        autoComp.minQueryLength = 2;
        autoComp.queryDelay = 0.5;
        autoComp.forceSelection = true;
        // @ts-expect-error Autocomp
        autoComp.itemSelectEvent.subscribe(function (_sType, args): void {
          const userId = args[1].getElementsByTagName('div').item(0).id.replace(/.*?([0-9]+)$/, "$1");
          const userName = args[1].getElementsByTagName('div').item(0).innerHTML;
          ManageSiteUserBlocksModule.listeners.selectUser(userId, userName);
        });

        // @ts-expect-error Autocomp
        autoComp.formatResult = function (aResultItem, _sQuery): string {
          const name = aResultItem[0];
          const userId = aResultItem[1];
          if (name != null) {
            return '<div id="user-autocomplete-' + userId + '">' + name + '</div>';
          } else {
            return "";
          }
        };
        new OZONE.forms.lengthLimiter("user-block-reason", "reason-char-left", 200);

        ManageSiteUserBlocksModule.vars.addFormInited = true;
      }
      document.getElementById("show-add-block-button")!.style.display = "none";
      document.getElementById("add-block-user-div")!.style.display = "block";
      OZONE.visuals.scrollTo("add-block-user-div");
    },
    cancelAdd: function (_event?: Event | null): void {
      // resets the forms?
      document.getElementById("show-add-block-button")!.style.display = "block";
      document.getElementById("add-block-user-div")!.style.display = "none";
      (<HTMLInputElement>document.getElementById("user-lookup")!).value = "";
      ManageSiteUserBlocksModule.listeners.changeUser(null);
    },
    selectUser: function (userId: number, userName: string): void {
      const userString = Wikijump.render.printuser(userId, userName, true);
      document.getElementById("select-user-div")!.style.display = "none";
      document.getElementById("selected-user-div")!.style.display = "block";
      document.getElementById("selected-user-rendered")!.innerHTML = userString;
      ManageSiteUserBlocksModule.vars.currentUserId = userId;
    },
    changeUser: function (_event?: Event | null): void {
      document.getElementById("select-user-div")!.style.display = "block";
      document.getElementById("selected-user-div")!.style.display = "none";
      (<HTMLInputElement>document.getElementById("user-lookup")!).value = "";
      ManageSiteUserBlocksModule.vars.currentUserId = null;
    },
    blockUser: function (_event?: Event | null): void {
      if (ManageSiteUserBlocksModule.vars.currentUserId == null) {
        const w = new OZONE.dialogs.ErrorDialog();
        w.content = "You must select a valid user to block.";
        w.show();
        return;
      }
      const params: RequestModuleParameters = {
        userId: ManageSiteUserBlocksModule.vars.currentUserId,
        reason: (<HTMLTextAreaElement>document.getElementById("user-block-reason")!).value,
        action: "ManageSiteBlockAction",
        event: "blockUser"
      };
      OZONE.ajax.requestModule(null, params, ManageSiteUserBlocksModule.callbacks.blockUser);
    },
    deleteBlock: function (_event: Event | null, userId: number, userName: string): void {
      const w = new OZONE.dialogs.ConfirmationDialog();
      w.buttons = ['cancel', 'yes, delete block'];
      w.addButtonListener('cancel', w.close);
      w.addButtonListener('yes, delete block', ManageSiteUserBlocksModule.listeners.deleteBlock2, userId);
      w.content = "Are you sure you want to remove the user block for the user <strong>" + userName + "</strong>?";
      w.show();
      ManageSiteUserBlocksModule.vars.dCurrentUserId = userId;
    },
    deleteBlock2: function (_event?: Event | null): void {
      const userId = ManageSiteUserBlocksModule.vars.dCurrentUserId;
      const params: RequestModuleParameters = {
        userId: userId,
        action: "ManageSiteBlockAction",
        event: "deleteBlock"
      };
      OZONE.ajax.requestModule(null, params, ManageSiteUserBlocksModule.callbacks.deleteBlock);
    }
  },
  callbacks: {
    blockUser: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }

      const w = new OZONE.dialogs.SuccessBox();
      w.content = "User blocked.";
      w.show();
      // refresh the screen too
      setTimeout(() => Wikijump.modules.ManageSiteModule.utils.loadModule("sm-user-blocks"), 1500);
    },
    deleteBlock: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }

      const w = new OZONE.dialogs.SuccessBox();
      w.content = "User block removed.";
      w.show();
      // refresh the screen too
      setTimeout(() => Wikijump.modules.ManageSiteModule.utils.loadModule("sm-user-blocks"), 1500);
    }
  }
};
