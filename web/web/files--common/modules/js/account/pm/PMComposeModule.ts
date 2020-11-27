import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
import { inboxPage } from "@Modules/account/AccountMessagesModule";
declare const YAHOO: any;
declare type YahooResponse = any;

export const PMComposeModule = {
  vars: {
    recipientId: null as null | number,
  },
  listeners: {
    changeRecipient: function (_event?: Event | null): void {
      document.getElementById("select-user-div")!.style.display = "block";
      document.getElementById("selected-user-div")!.style.display = "none";
      (<HTMLInputElement>document.getElementById("user-lookup")!).value = "";
      PMComposeModule.vars.recipientId = null;
    },
    preview: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        source: (<HTMLTextAreaElement>document.getElementById("editor-textarea")!).value,
        subject: (<HTMLInputElement>document.getElementById("pm-subject")!).value
      };
      if (PMComposeModule.vars.recipientId) {
        params.to_user_id = PMComposeModule.vars.recipientId;
      }
      OZONE.ajax.requestModule("account/pm/PMPreviewModule", params, PMComposeModule.callbacks.preview);
    },
    saveDraft: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        source: (<HTMLTextAreaElement>document.getElementById("editor-textarea")!).value,
        subject: (<HTMLInputElement>document.getElementById("pm-subject")!).value,
        action: "PMAction",
        event: "saveDraft"
      };
      if (PMComposeModule.vars.recipientId != null) {
        params.to_user_id = PMComposeModule.vars.recipientId;
      }
      OZONE.ajax.requestModule("Empty", params, PMComposeModule.callbacks.saveDraft);
    },
    send: function (_event?: Event | null): void {
      if (PMComposeModule.vars.recipientId == null) {
        // no recipient!
        const d = new OZONE.dialogs.ErrorDialog();
        d.content = "The recipient of the message should be chosen ;-)";
        d.show();
        return;
      }
      const params: RequestModuleParameters = {
        source: (<HTMLTextAreaElement>document.getElementById("editor-textarea")!).value,
        subject: (<HTMLInputElement>document.getElementById("pm-subject")!).value,
        to_user_id: PMComposeModule.vars.recipientId,
        action: "PMAction",
        event: "send"
      };
      OZONE.ajax.requestModule("Empty", params, PMComposeModule.callbacks.send);
    },
    cancel: function (_event?: Event | null): void {
      // warning: need to check the context... is it a "reply" or standalone compose?
      Wikijump.Editor.shutDown();
    },
    showContactsList: function (_event?: Event | null): void {
      OZONE.ajax.requestModule("account/pm/PMComposeContactsListModule", {}, PMComposeModule.callbacks.showContactsList);
    }
  },
  callbacks: {
    preview: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      OZONE.utils.setInnerHTMLContent("pm-preview-area", response.body);
      OZONE.visuals.scrollTo("pm-preview-area");
    },
    send: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const w = new OZONE.dialogs.SuccessBox();
      w.content = "Message has been sent.";
      w.show();
      setTimeout(() => inboxPage(), 1500);
    },
    saveDraft: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const w = new OZONE.dialogs.SuccessBox();
      w.content = "Draft has been saved.";
      w.show();
    },
    checkCan: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
    },
    showContactsList: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const w = new OZONE.dialogs.Dialog();
      w.content = response.body;
      w.show();

      // check for sizes
      const list = document.getElementById("pm-contacts-list")!;
      if (list.offsetHeight > 500) {
        list.style.height = "500px";
        list.style.overflow = "auto";

        OZONE.dialog.factory.boxcontainer().centerContent();
      }
    }
  },
  init: function (): void {
    // init autocomplete
    const dataSource = new YAHOO.widget.DS_XHR("/quickmodule.php", ['users', 'name', 'user_id']);
    dataSource.scriptQueryParam = "q";
    dataSource.scriptQueryAppend = "&module=UserLookupQModule";

    const autoComp = new YAHOO.widget.AutoComplete('user-lookup', 'user-lookup-list', dataSource);

    autoComp.minQueryLength = 2;
    autoComp.queryDelay = 0.5;
    autoComp.forceSelection = true;
    //@ts-expect-error Autocomp
    autoComp.itemSelectEvent.subscribe(function (_sType, args): void {
      const userId = args[1].getElementsByTagName('div').item(0).id.replace(/.*?([0-9]+)$/, "$1");
      const userName = args[1].getElementsByTagName('div').item(0).innerHTML;
      PMComposeModule.utils.selectRecipient(userId, userName);
    });

    //@ts-expect-error Autocomp
    autoComp.formatResult = function (aResultItem, _sQuery): string {
      const name = aResultItem[0];
      const userId = aResultItem[1];
      if (name != null) {
        return `<div id="user-autocomplete-${userId}">${name}</div>`;
      } else {
        return "";
      }
    };

    if (Wikijump.modules.AccountMessagesModule.vars.toUserId) {
      PMComposeModule.utils.selectRecipient(
        Wikijump.modules.AccountMessagesModule.vars.toUserId!,
        Wikijump.modules.AccountMessagesModule.vars.toUserName!
      );
      Wikijump.modules.AccountMessagesModule.vars.toUserId = null;
      Wikijump.modules.AccountMessagesModule.vars.toUserName = null;
    }
    // init editor
    Wikijump.Editor.init("editor-textarea", "editor-panel");

    YAHOO.util.Event.addListener("pm-compose-cancel-button", "click", PMComposeModule.listeners.cancel);
  },
  utils: {
    selectRecipient: function (userId: number, userName: string): void {
      const userString = Wikijump.render.printuser(userId, userName, true);
      document.getElementById("select-user-div")!.style.display = "none";
      document.getElementById("selected-user-div")!.style.display = "block";
      document.getElementById("selected-user-rendered")!.innerHTML = userString;
      PMComposeModule.vars.recipientId = userId;

      // also check for permission
      const params: RequestModuleParameters = {
        userId: userId,
        action: "PMAction",
        event: "checkCan"
      };
      OZONE.ajax.requestModule(null, params, PMComposeModule.callbacks.checkCan);
    }
  }
};

PMComposeModule.init();
