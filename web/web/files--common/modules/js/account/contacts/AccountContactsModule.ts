import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare const YAHOO: any;
declare type YahooResponse = any;
declare const fx: any;

export const AccountContactsModule = {
  vars: {
    addFormInited: false,
    currentUserId: null as null | number,
  },
  listeners: {
    showAddForm: function (_event?: Event | null): void {
      if (!AccountContactsModule.vars.addFormInited) {
        // init autocomplete now
        const dataSource = new YAHOO.widget.DS_XHR("/quickmodule.php", ['users', 'name', 'user_id']);
        dataSource.scriptQueryParam = "q";
        dataSource.scriptQueryAppend = "&module=UserLookupQModule";

        const autoComp = new YAHOO.widget.AutoComplete('user-lookup', 'user-lookup-list', dataSource);

        autoComp.minQueryLength = 2;
        autoComp.queryDelay = 0.5;
        autoComp.forceSelection = true;
        // @ts-expect-error Yahoo typings are a mystery
        autoComp.itemSelectEvent.subscribe(function (_sType, args): void {
          const userId = args[1].getElementsByTagName('div').item(0).id.replace(/.*?([0-9]+)$/, "$1");
          const userName = args[1].getElementsByTagName('div').item(0).innerHTML;
          AccountContactsModule.listeners.selectUser(userId, userName);
        });

        // @ts-expect-error Yahoo typings are a mystery
        autoComp.formatResult = function (aResultItem, _sQuery): string {
          const name = aResultItem[0];
          const userId = aResultItem[1];
          if (name != null) {
            return '<div id="user-autocomplete-' + userId + '">' + name + '</div>';
          } else {
            return "";
          }
        };
        AccountContactsModule.vars.addFormInited = true;
      }
      document.getElementById("show-add-contact-button")!.style.display = "none";
      document.getElementById("add-contact-user-div")!.style.display = "block";
      OZONE.visuals.scrollTo("add-contact-user-div");
    },
    cancelAdd: function (_event?: Event | null): void {
      // resets the forms?
      document.getElementById("show-add-contact-button")!.style.display = "block";
      document.getElementById("add-contact-user-div")!.style.display = "none";
      (<HTMLInputElement>document.getElementById("user-lookup")).value = "";
      AccountContactsModule.listeners.changeUser(null);
    },
    selectUser: function (userId: number, userName: string): void {
      const userString = Wikijump.render.printuser(userId, userName, true);
      document.getElementById("select-user-div")!.style.display = "none";
      document.getElementById("selected-user-div")!.style.display = "block";
      document.getElementById("selected-user-rendered")!.innerHTML = userString;
      AccountContactsModule.vars.currentUserId = userId;
    },
    changeUser: function (_event?: Event | null): void {
      document.getElementById("select-user-div")!.style.display = "block";
      document.getElementById("selected-user-div")!.style.display = "none";
      (<HTMLInputElement>document.getElementById("user-lookup")).value = "";
      AccountContactsModule.vars.currentUserId = null;
    },

    addContact: function (_event?: Event | null): void {
      if (AccountContactsModule.vars.currentUserId == null) {
        const w = new OZONE.dialogs.ErrorDialog();
        w.content = "You must select a valid user to add.";
        w.show();
        return;
      }
      OZONE.ajax.requestModule('userinfo/UserAddToContactsModule', { userId: AccountContactsModule.vars.currentUserId }, AccountContactsModule.callbacks.addContact);
    },

    showBack: function (_event?: Event | null): void {
      OZONE.ajax.requestModule("account/contacts/AccountBackContactsModule", {}, AccountContactsModule.callbacks.showBack);
    },

    removeContact: function (_event: Event | null, userId: number): void {
      const params: RequestModuleParameters = {};
      params.action = "ContactsAction";
      params.event = "removeContact";
      params.userId = userId;
      OZONE.ajax.requestModule(null, params, AccountContactsModule.callbacks.removeContact);
    },

    refresh: function (_event?: Event | null): void {
      Wikijump.modules.AccountModule.utils.loadModule("am-contacts");
    }
  },
  callbacks: {
    showBack: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      document.getElementById("back-contacts-list")!.innerHTML = response.body;
    },
    addContact: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const w = new OZONE.dialogs.Dialog();
      w.content = response.body;
      w.show();
    },
    removeContact: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const w = new OZONE.dialogs.SuccessBox();
      w.content = "User removed from contacts";
      w.show();
      AccountContactsModule.listeners.refresh();
    }
  }
};
