import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare const YAHOO: any;
declare type YahooResponse = any;

// Defined in templates/modules/account/AccountModule.tpl
declare const composeTo: string | undefined;
declare const inboxMessage: string | undefined;

export const AccountMessagesModule = {
  init: function (): void {
    if (composeTo) {
      AccountMessagesModule.listeners.compose(null, composeTo);
      // attach cancel listener ???
    } else if (inboxMessage) {
      AccountMessagesModule.listeners.viewInboxMessage(inboxMessage);
    } else {
      inboxPage();
    }
  },
  vars: {
    toUserId: null,
    toUserName: null,
  },
  listeners: {
    inbox: function (_event: Event | null, pageNo?: number): void {
      const params: RequestModuleParameters = {};
      if (pageNo) {
        params.page = pageNo;
      }
      OZONE.ajax.requestModule("account/pm/PMInboxModule", params, AccountMessagesModule.callbacks.setActionArea);
      const tp = document.getElementById("account-top-tabs")!;
      const as = tp.getElementsByTagName('a');
      for (let i = 0; i < as.length; i++) {
        YAHOO.util.Dom.removeClass(as[i], "active");
      }
      const curr = as.item(0);

      YAHOO.util.Dom.addClass(curr, "active");
    },

    sent: function (_event: Event | null, pageNo?: number): void {
      const params: RequestModuleParameters = {};
      if (pageNo) {
        params.page = pageNo;
      }
      OZONE.ajax.requestModule("account/pm/PMSentModule", params, AccountMessagesModule.callbacks.setActionArea);
      const tp = document.getElementById("account-top-tabs")!;
      const as = tp.getElementsByTagName('a');
      for (let i = 0; i < as.length; i++) {
        YAHOO.util.Dom.removeClass(as[i], "active");
      }
      const curr = as.item(1);
      YAHOO.util.Dom.addClass(curr, "active");
    },
    drafts: function (_event: Event | null, pageNo?: number): void {
      let params: RequestModuleParameters = {};
      if (pageNo) { params = { page: pageNo }; }
      OZONE.ajax.requestModule("account/pm/PMDraftsModule", params, AccountMessagesModule.callbacks.setActionArea);
      const tp = document.getElementById("account-top-tabs")!;
      const as = tp.getElementsByTagName('a');
      for (let i = 0; i < as.length; i++) {
        YAHOO.util.Dom.removeClass(as[i], "active");
      }
      const curr = as.item(2);
      YAHOO.util.Dom.addClass(curr, "active");
    },

    compose: function (_event: Event | null, userId: string): void {
      const params: RequestModuleParameters = {};
      if (userId != null) {
        params.toUserId = userId;
      }
      OZONE.ajax.requestModule("account/pm/PMComposeModule", params, AccountMessagesModule.callbacks.compose);
      const tp = document.getElementById("account-top-tabs")!;
      const as = tp.getElementsByTagName('a');
      for (let i = 0; i < as.length; i++) {
        YAHOO.util.Dom.removeClass(as[i], "active");
      }
      const curr = as.item(3);
      YAHOO.util.Dom.addClass(curr, "active");

      YAHOO.util.Event.addListener("pm-compose-cancel-button", "click", AccountMessagesModule.listeners.inbox);
    },

    viewInboxMessage: function (messageId: string): void {
      const params: RequestModuleParameters = {};
      params.message_id = messageId;
      OZONE.ajax.requestModule("account/pm/PMInboxMessageModule", params, AccountMessagesModule.callbacks.setActionArea);
    },
    replyInboxMessage: function (_event: Event | null, messageId: string): void {
      const params: RequestModuleParameters = {};
      if (messageId) {
        params.replyMessageId = messageId;
      }
      OZONE.ajax.requestModule("account/pm/PMComposeModule", params, AccountMessagesModule.callbacks.replyInboxMessage);
    },

    cancelReplyInboxMessage: function (_event: Event | null): void {
      document.getElementById("pm-reply-area")!.innerHTML = "";
      document.getElementById("inbox-message-options")!.style.display = "block";
      const nav1 = document.getElementById("inbox-message-nav")!;
      if (nav1) {
        nav1.style.display = "block";
      }
    },
    viewSentMessage: function (messageId: string): void {
      const params: RequestModuleParameters = {};
      params.message_id = messageId;
      OZONE.ajax.requestModule("account/pm/PMSentMessageModule", params, AccountMessagesModule.callbacks.setActionArea);
    },
    viewDraftsMessage: function (messageId: string): void {
      const params: RequestModuleParameters = {};
      params.message_id = messageId;
      OZONE.ajax.requestModule("account/pm/PMDraftsMessageModule", params, AccountMessagesModule.callbacks.setActionArea);
    }
  },
  callbacks: {
    setActionArea: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      document.getElementById("pm-action-area")!.innerHTML = response.body;
      // format dates
      OZONE.utils.formatDates(document.getElementById("pm-action-area")!);
      Wikijump.page.fixers.fixEmails(document.getElementById("pm-action-area")!);
    },
    compose: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      document.getElementById("pm-action-area")!.innerHTML = response.body;
      // format dates
      OZONE.utils.formatDates(document.getElementById("pm-action-area")!);
      if (response.toUserId) {
        AccountMessagesModule.vars.toUserId = response.toUserId;
        AccountMessagesModule.vars.toUserName = response.toUserName;
      } else {
        AccountMessagesModule.vars.toUserId = null;
        AccountMessagesModule.vars.toUserName = null;
      }
    },
    replyInboxMessage: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      if (response.toUserId) {
        AccountMessagesModule.vars.toUserId = response.toUserId;
        AccountMessagesModule.vars.toUserName = response.toUserName;
      } else {
        AccountMessagesModule.vars.toUserId = null;
        AccountMessagesModule.vars.toUserName = null;
      }
      document.getElementById("pm-reply-area")!.innerHTML = response.body;
      setTimeout('OZONE.visuals.scrollTo(document.getElementById("pm-reply-area")!)', 200);
      document.getElementById("inbox-message-options")!.style.display = "none";
      const nav1 = document.getElementById("inbox-message-nav")!;
      if (nav1) {
        nav1.style.display = "none";
      }

      YAHOO.util.Event.addListener("pm-compose-cancel-button", "click", AccountMessagesModule.listeners.cancelReplyInboxMessage);
    },
  },
  utils: {
    highlightTab: function (event: Event): void {
      // dehighlight all tabs
      const tp = document.getElementById("account-top-tabs")!;
      const as = tp.getElementsByTagName('a');
      for (let i = 0; i < as.length; i++) {
        YAHOO.util.Dom.removeClass(as[i], "active");
      }
      const curr = YAHOO.util.Event.getTarget(event);
      if (curr.tagName.toLowerCase() == 'a') {
        YAHOO.util.Dom.addClass(curr, "active");
      }
    }
  }
};

export function inboxPage (pageNo?: number): void {
  AccountMessagesModule.listeners.inbox(null, pageNo);
}

export function sentPage (pageNo: number): void {
  AccountMessagesModule.listeners.sent(null, pageNo);
}
export function draftsPage (pageNo: number): void {
  AccountMessagesModule.listeners.drafts(null, pageNo);
}


AccountMessagesModule.init();
