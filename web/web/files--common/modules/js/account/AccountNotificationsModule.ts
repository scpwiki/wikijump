import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare type YahooResponse = any;

export const AccountNotificationsModule = {
  listeners: {
    loadList: function (_event: Event | null, pageNo: number): void {
      let params: RequestModuleParameters = {};
      if (pageNo) {
        params = { page: pageNo };
      }
      OZONE.ajax.requestModule("account/AccountNotificationsListModule", params, AccountNotificationsModule.callbacks.loadList);
    }
  },
  callbacks: {
    loadList: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      document.getElementById("notifications-area")!.innerHTML = response.body;
      OZONE.utils.formatDates(document.getElementById("notifications-area")!);
    }
  }
};
