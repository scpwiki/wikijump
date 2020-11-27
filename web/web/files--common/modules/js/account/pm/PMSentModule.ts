import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
import { sentPage } from "@Modules/account/AccountMessagesModule";
declare const YAHOO: any;
declare type YahooResponse = any;

export const PMSentModule = {
  vars: {
    currentMessageId: null as null | number
  },
  listeners: {
    loadList: function (event: Event | null, pageNo: number): void {
      let params: RequestModuleParameters = {};
      if (pageNo) { params = { page: pageNo }; }
      OZONE.ajax.requestModule("account/pm/PMSentModule", params, Wikijump.modules.AccountMessagesModule.callbacks.setActionArea);
      if (event) {	Wikijump.modules.AccountMessagesModule.utils.highlightTab(event); }
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
        event: 'removeSelectedSent',
        selected: JSON.stringify(selected)
      };
      OZONE.ajax.requestModule(null, params, PMSentModule.callbacks.removeSelected);
    },
    removeSentMessage: function (_event: Event | null, messageId: number): void {
      PMSentModule.vars.currentMessageId = messageId;
      const w = new OZONE.dialogs.ConfirmationDialog();
      w.content = "Are sure you want to remove this message?";
      w.buttons = ['cancel', 'remove message'];
      w.addButtonListener('cancel', w.close);
      w.addButtonListener('remove message', PMSentModule.listeners.removeSentMessage2);
      w.focusButton = 'cancel';
      w.show();
    },
    removeSentMessage2: function (_event?: Event | null, _messageId?: number): void {
      const params: RequestModuleParameters = {
        action: "PMAction",
        event: 'removeSentMessage',
        message_id: PMSentModule.vars.currentMessageId
      };
      OZONE.ajax.requestModule(null, params, PMSentModule.callbacks.removeSentMessage);
    }
  },
  callbacks: {
    removeSelected: function (_response: YahooResponse): void {
      PMSentModule.listeners.loadList(null, 1);
    },
    removeSentMessage: function (response: YahooResponse): void {
      if (response.status == 'ok') {
        const w = new OZONE.dialogs.SuccessBox();
        w.content = "The message has been removed.";
        w.show();

        if (response.messageId) {
          setTimeout(() => Wikijump.modules.AccountMessagesModule.listeners.viewSentMessage("' + response.messageId + '"), 1000);
        } else {
          // return to inbox view
          setTimeout(() => sentPage(1));
        }
      }
    }
  }
};
