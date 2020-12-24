import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare const YAHOO: any;
declare type YahooResponse = any;

export const ManageSiteEmailListsModule = {
  vars: {
    currentStatus: null as null | 'new' | 'edit',
    currentContainer: null as null | HTMLElement,
    currentListId: null as null | number
  },
  listeners: {
    clickNewList: function (_event?: Event | null): void {
      if (ManageSiteEmailListsModule.vars.currentStatus) {
        return;
      }
      const form = document.getElementById('elist-form-template')!;
      const form2 = <HTMLElement>form.cloneNode(true);
      const aa = document.getElementById('elist-action-area')!;
      // clear aa
      aa.innerHTML = '';
      form2.id = "elist-new-list-form";
      aa.appendChild(form2);
      document.getElementById("elist-add-new-button")!.style.display = "none";
      ManageSiteEmailListsModule.vars.currentStatus = 'new';

      YAHOO.util.Event.addListener(form2, 'submit', ManageSiteEmailListsModule.listeners.saveList);
    },
    closeEditList: function (_event?: Event | null): void {
      if (ManageSiteEmailListsModule.vars.currentStatus == 'edit') {
        const c = ManageSiteEmailListsModule.vars.currentContainer!;
        c.parentNode!.removeChild(c);
      }
      if (ManageSiteEmailListsModule.vars.currentStatus == 'new') {
        const aa = document.getElementById('elist-action-area')!;
        aa.innerHTML = '';
        document.getElementById("elist-add-new-button")!.style.display = "block";
      }
      ManageSiteEmailListsModule.vars = {
        currentStatus: null,
        currentContainer: null,
        currentListId: null
      };
    },
    removeList: function (_event?: Event | null): void {
      alert("List removal is not implemented yet.");
    },
    editList: function (_event: Event | null, listId: number): void {
      if (ManageSiteEmailListsModule.vars.currentStatus) {
        return;
      }
      const row = document.getElementById('elist-row-' + listId)!;
      const isSpecial = YAHOO.util.Dom.hasClass(row, 'elist-special');
      const title = YAHOO.util.Dom.getElementsByClassName('l-title', 'span', row)[0].innerHTML;
      const unixName = YAHOO.util.Dom.getElementsByClassName('l-unixname', 'span', row)[0].innerHTML;
      const whoCanJoin = YAHOO.util.Dom.getElementsByClassName('l-whocanjoin', 'span', row)[0].innerHTML;

      // add container
      const tr = document.createElement('tr');
      const td = document.createElement('td');
      tr.appendChild(td);
      td.colSpan = 4;

      const form2 = <HTMLElement>document.getElementById('elist-form-template')!.cloneNode(true);
      form2.id = "elist-edit";
      const inputs = form2.getElementsByTagName('input');
      inputs[1].value = unixName;
      inputs[0].value = title;
      form2.getElementsByTagName('select')[0].value = whoCanJoin;
      td.appendChild(form2);

      if (isSpecial) {
        inputs[1].disabled = true;
        form2.getElementsByTagName('select')[0].disabled = true;
      }

      OZONE.dom.insertAfter(row.parentNode!, tr, row);

      ManageSiteEmailListsModule.vars.currentStatus = 'edit';
      ManageSiteEmailListsModule.vars.currentContainer = tr;
      ManageSiteEmailListsModule.vars.currentListId = listId;

      YAHOO.util.Event.addListener(form2, 'submit', ManageSiteEmailListsModule.listeners.saveList);
    },
    embedInfo: function (_event: Event | null, listId: number): void {
      ManageSiteEmailListsModule.listeners.closeEmbedInfo();
      const row = document.getElementById('elist-row-' + listId)!;
      // add container
      const tr = document.createElement('tr');
      tr.className = 'elist-embedinfo-row';
      const td = document.createElement('td');
      tr.appendChild(td);
      td.colSpan = 4;

      const tt = document.getElementById('elist-embed-template')!.cloneNode(true);
      td.appendChild(tt);
      OZONE.dom.insertAfter(row.parentNode!, tr, row);

      const ll = YAHOO.util.Dom.getElementsByClassName('l-unixname', 'span', tt)[0];
      const unixName = YAHOO.util.Dom.getElementsByClassName('l-unixname', 'span', row)[0].innerHTML;
      ll.innerHTML = unixName;
    },
    closeEmbedInfo: function (_event?: Event | null): void {
      const cs = YAHOO.util.Dom.getElementsByClassName('elist-embedinfo-row');
      for (let i = 0; i < cs.length; i++) {
        cs[i].parentNode.removeChild(cs[i]);
      }
    },
    saveList: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        // @ts-expect-error What is this?
        ...OZONE.utils.formToArray(this),
        action: 'ManageSiteEmailListsAction',
        event: 'saveList',
      };
      if (ManageSiteEmailListsModule.vars.currentListId) {
        params.listId = ManageSiteEmailListsModule.vars.currentListId;
      }
      OZONE.ajax.requestModule(null, params, ManageSiteEmailListsModule.callbacks.saveList);
    },
    showSubscribers: function (_event: Event | null, listId: number): void {
      const params: RequestModuleParameters = { listId };
      OZONE.ajax.requestModule("managesite/elists/ManageSiteEmailListSubscribersModule", params, ManageSiteEmailListsModule.callbacks.showSubscribers);
    },
    reloadMain: function (_event?: Event | null): void {
      Wikijump.modules.ManageSiteModule.utils.loadModule('sm-email-lists');
    },
    removeSubscriber: function (_event: Event | null, userId: number, listId: number): void {
      const params: RequestModuleParameters = {
        userId,
        listId,
        action: 'ManageSiteEmailListsAction',
        event: 'unsubscribe'
      };
      OZONE.ajax.requestModule("managesite/elists/ManageSiteEmailListSubscribersModule", params, ManageSiteEmailListsModule.callbacks.showSubscribers);
    }
  },
  callbacks: {
    saveList: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }

      Wikijump.modules.ManageSiteModule.utils.loadModule('sm-email-lists');
    },
    showSubscribers: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      document.getElementById("sm-action-area")!.innerHTML = response.body;
    }
  }
};
