import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
import { draftsPage } from "@Modules/account/AccountMessagesModule";
declare const YAHOO: any;
declare type YahooResponse = any;

export const PMDraftsModule = {
  vars: {
    currentMessageId: null as null | number,
  },
  listeners: {
    loadList: function (event: Event | null, pageNo: number): void {
      let params: RequestModuleParameters = {};
      if (pageNo) {
        params = { page: pageNo };
      }
      OZONE.ajax.requestModule("account/pm/PMDraftsModule", params, Wikijump.modules.AccountMessagesModule.callbacks.setActionArea);
      if (event) {
        Wikijump.modules.AccountMessagesModule.utils.highlightTab(event);
      }
    },
    selectAll: function (_event?: Event | null): void {
      const chs = YAHOO.util.Dom.getElementsByClassName("message-select");
      for (let i = 0; i < chs.length; i++) {
        chs[i].checked = true;
      }
    },
    removeSelected: function (_event?: Event | null): void {
      const selected = [];
      const chs = YAHOO.util.Dom.getElementsByClassName("message-select");
      for (let i = 0; i < chs.length; i++) {
        if (chs[i].checked) {
          selected.push(chs[i].id.replace(/message-check-/, ''));
        }
      }
      if (selected.length == 0) {
        return;
      }
      const params: RequestModuleParameters = {
        action: "PMAction",
        event: 'removeSelectedDrafts',
        selected: JSON.stringify(selected)
      };
      OZONE.ajax.requestModule(null, params, PMDraftsModule.callbacks.removeSelected);
    },
    removeDraftsMessage: function (_event: Event | null, messageId: number): void {
      PMDraftsModule.vars.currentMessageId = messageId;
      const w = new OZONE.dialogs.ConfirmationDialog();
      w.content = "Are sure you want to remove this message?";
      w.buttons = ['cancel', 'remove message'];
      w.addButtonListener('cancel', w.close);
      w.addButtonListener('remove message', PMDraftsModule.listeners.removeDraftsMessage2);
      w.focusButton = 'cancel';
      w.show();
    },
    removeDraftsMessage2: function (_event?: Event | null, _messageId?: number): void {
      const params: RequestModuleParameters = {
        action: "PMAction",
        event: 'removeDraftsMessage',
        message_id: PMDraftsModule.vars.currentMessageId
      };
      OZONE.ajax.requestModule(null, params, PMDraftsModule.callbacks.removeDraftsMessage);
    },
    editDraftMessage: function (_event: Event | null, messageId: number): void {
      const params: RequestModuleParameters = {};
      if (messageId) {
        params.continueMessageId = messageId;
      }
      OZONE.ajax.requestModule("account/pm/PMComposeModule", params, PMDraftsModule.callbacks.editDraftMessage);
    }
  },
  callbacks: {
    removeSelected: function (_response?: YahooResponse): void {
      PMDraftsModule.listeners.loadList(null, 1);
    },
    removeDraftsMessage: function (response: YahooResponse): void {
      if (response.status == 'ok') {
        const w = new OZONE.dialogs.SuccessBox();
        w.content = "The message has been removed.";
        w.show();

        if (response.messageId) {
          setTimeout(() => Wikijump.modules.AccountMessagesModule.listeners.viewDraftsMessage("' + response.messageId + '"), 1000);
        } else {
          // return to inbox view
          setTimeout(() => draftsPage(1));
        }
      }
    },
    editDraftMessage: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      if (response.toUserId) {
        Wikijump.modules.AccountMessagesModule.vars.toUserId = response.toUserId;
        Wikijump.modules.AccountMessagesModule.vars.toUserName = response.toUserName;
      } else {
        Wikijump.modules.AccountMessagesModule.vars.toUserId = null;
        Wikijump.modules.AccountMessagesModule.vars.toUserName = null;
      }
      document.getElementById("pm-action-area")!.innerHTML = response.body;
      // format dates
      OZONE.utils.formatDates(document.getElementById("pm-action-area")!);

      const tp = document.getElementById("account-top-tabs")!;
      const as = tp.getElementsByTagName('a');
      for (let i = 0; i < as.length; i++) {
        YAHOO.util.Dom.removeClass(as[i], "active");
      }
      const curr = as.item(3);
      YAHOO.util.Dom.addClass(curr, "active");

      YAHOO.util.Event.addListener("pm-compose-cancel-button", "click", Wikijump.modules.AccountMessagesModule.listeners.drafts);
    }
  }
};
