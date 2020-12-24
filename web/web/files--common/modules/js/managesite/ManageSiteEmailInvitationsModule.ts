import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare const YAHOO: any;
declare type YahooResponse = any;

export const ManageSiteEmailInvitationsModule = {
  vars: {
    addresses: [] as [string, string, boolean][]
  },
  listeners: {
    moreRecipients: function (_event?: Event | null, rname?: string, email?: string): void {
      // if (document.getElementById("invitation-addresses")!.getElementsByTagName("table").length > 200) {
      //   let w = new OZONE.dialogs.ErrorDialog();
      //   w.content = "Sorry, you cannot send more than 200 invitations at once.";
      //   w.show();
      //   return;
      // }
      const temp = document.getElementById("recipient-template")!.getElementsByTagName('table')[0];
      const clone = <HTMLElement>temp.cloneNode(true);
      if (rname || email) {
        const inpts = clone.getElementsByTagName('input');
        inpts[0].value = rname!;
        inpts[1].value = email!;
      }
      document.getElementById("invitation-addresses")!.appendChild(clone);
    },
    removeRecipient: function (event: Event): void {
      // get the parrent "table" element
      let el = YAHOO.util.Event.getTarget(event);
      while (el && el.tagName.toLowerCase() != "table") {
        el = el.parentNode;
      }
      if (el) {
        el.parentNode.removeChild(el);
      }
      ManageSiteEmailInvitationsModule.listeners.updateTo();
    },
    updateTo: function (_event?: Event | null): void {
      ManageSiteEmailInvitationsModule.utils.updateAddresses(null);
      const adrs = ManageSiteEmailInvitationsModule.vars.addresses;
      if (!adrs) { return; }
      const frmt = [];

      for (let i = 0; i < adrs.length; i++) {
        frmt.push(adrs[i][1] + ' <' + adrs[i][0] + '>');
      }

      document.getElementById("recipients-list-formatted")!.innerHTML = OZONE.utils.escapeHtml(frmt.join(', '));
    },
    send: function (_event?: Event | null): void {
      ManageSiteEmailInvitationsModule.utils.updateAddresses(null);
      const adrs = ManageSiteEmailInvitationsModule.vars.addresses;

      if (adrs.length == 0) {
        const w = new OZONE.dialogs.ErrorDialog();
        w.content = "No valid recepients have been given. For each person both the email address and name should be given.";
        w.show();
        return;
      }

      // check for invalid records
      const invalids = YAHOO.util.Dom.getElementsByClassName('invalid', 'input', document.getElementById("invitation-addresses")!);
      if (invalids.length > 0) {
        if (!confirm("The list contains incomplete entries. Are you sure you want to continue? \nOnly valid entries will be used to send invitations if you continue.")) {
          return;
        }
      }
      // make a confirmation? no.

      const serialized = JSON.stringify(adrs);
      const params: RequestModuleParameters = {
        addresses: serialized,
        event: 'sendEmailInvitations',
        action: 'ManageSiteMembershipAction',
        message: (<HTMLTextAreaElement>document.getElementById("inv-message")!).value
      };
      OZONE.ajax.requestModule(null, params, ManageSiteEmailInvitationsModule.callbacks.send);

      const w = new OZONE.dialogs.WaitBox();
      w.content = "Sending invitations...";
      w.show();
    },
    showBulkAdd: function (_event?: Event | null): void {
      document.getElementById("invitation-addresses-upload-box")!.style.display = "none";
      document.getElementById("invitation-addresses-bulk-box")!.style.display = "block";
      OZONE.visuals.scrollTo("invitation-addresses-bulk-box");
    },
    cancelBulkAdd: function (_event?: Event | null): void {
      document.getElementById("invitation-addresses-bulk-box")!.style.display = "none";
    },
    processBulkAdd: function (_event?: Event | null): void {
      const text = (<HTMLTextAreaElement>document.getElementById("invitation-addresses-bulk-text")!).value;
      const entries = text.split(/[\n,]+/);
      let email, rname;
      const eReg = /[a-z0-9._-]+@[a-z0-9-]+(\.[a-z0-9-]+)+/i;
      for (let i = 0; i < entries.length; i++) {
        if (eReg.test(entries[i])) {
          email = eReg.exec(entries[i])![0];
          rname = entries[i].replace(eReg, '');
          rname = rname.replace(/[<>]/g, ' ');
          rname = rname.replace(/ +/g, ' ');
          rname = rname.replace(/^ +/, ' ');
          rname = rname.replace(/ +$/, ' ');
          ManageSiteEmailInvitationsModule.listeners.moreRecipients(null, rname, email);
        }
      }
      document.getElementById("invitation-addresses-bulk-box")!.style.display = "none";
      (<HTMLTextAreaElement>document.getElementById("invitation-addresses-bulk-text")!).value = '';
      ManageSiteEmailInvitationsModule.listeners.tidyList(null);
    },
    showUpload: function (_event?: Event | null): void {
      document.getElementById("invitation-addresses-bulk-box")!.style.display = "none";
      document.getElementById("invitation-addresses-upload-box")!.style.display = "block";
      OZONE.visuals.scrollTo("invitation-addresses-upload-box");
    },
    cancelUpload: function (_event?: Event | null): void {
      document.getElementById("invitation-addresses-upload-box")!.style.display = "none";
    },
    setAllToContacts: function (_event: Event | null, value: boolean): void {
      const tbls = document.getElementById("invitation-addresses")!.getElementsByTagName("table");
      for (let i = 0; i < tbls.length; i++) {
        const inpts = tbls[i].getElementsByTagName('input');
        inpts[2].checked = value;
      }
    },
    tidyList: function (_event?: Event | null): void {
      // remove empty elements, remove duplicates, add a few empty at the end
      const tbls = document.getElementById("invitation-addresses")!.getElementsByTagName("table");
      const emails: string[] = [];
      const toRemove = [];
      for (let i = 0; i < tbls.length; i++) {
        // get values
        const inpts = tbls[i].getElementsByTagName('input');
        const email = inpts[1].value;
        if (email == '') {
          toRemove.push(tbls[i]);
        }
        // check if email already in the emails
        for (let j = 0; j < emails.length; j++) {
          if (email == emails[j]) {
            toRemove.push(tbls[i]);
            break;
          }
        }
        emails.push(email);
      }

      for (let i = 0; i < toRemove.length; i++) {
        if (toRemove[i].parentNode) {
          toRemove[i].parentNode!.removeChild(toRemove[i]);
        }
      }

      ManageSiteEmailInvitationsModule.listeners.updateTo(null);

      ManageSiteEmailInvitationsModule.listeners.moreRecipients();
      ManageSiteEmailInvitationsModule.listeners.moreRecipients();
    },
    startUpload: function (_event?: Event | null): void {
      return;
    },
    contactsUploaded: function (status: string, addr: string): void {
      if (status != "ok") {
        const w = new OZONE.dialogs.ErrorDialog();
        w.content = "Error uploading the file.";
        w.show();
        return;
      }

      const ads = JSON.parse(addr);

      for (let i = 0; i < ads.length; i++) {
        ManageSiteEmailInvitationsModule.listeners.moreRecipients(null, ads[i].name, ads[i].email);
      }

      document.getElementById("invitation-addresses-upload-box")!.style.display = "none";

      ManageSiteEmailInvitationsModule.listeners.tidyList(null);
    }
  },
  callbacks: {
    send: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }

      const w = new OZONE.dialogs.SuccessBox();
      w.content = "Invitations have been saved";
      w.show();

      window.setTimeout(() => {
        Wikijump.modules.ManageSiteModule.utils.loadModule('sm-invitations-history');
        OZONE.visuals.scrollTo('header');
      }, 1000);
    }
  },
  utils: {
    updateAddresses: function (_event?: Event | null): void {
      // manually create a list of addresses
      const adrs: [string, string, boolean][] = [];
      const tbls = document.getElementById("invitation-addresses")!.getElementsByTagName("table");
      for (let i = 0; i < tbls.length; i++) {
        // get values
        const inpts = tbls[i].getElementsByTagName('input');
        const email = inpts[1].value;
        if (email != '' && !email.match(/^[a-z0-9._-]+@[a-z0-9-]+(\.[a-z0-9-]+)+$/i)) {
          YAHOO.util.Dom.addClass(inpts[1], 'invalid');
        } else {
          YAHOO.util.Dom.removeClass(inpts[1], 'invalid');
        }
        const rname = inpts[0].value;
        if (email != '' && email.match(/^[a-z0-9._-]+@[a-z0-9-]+(\.[a-z0-9-]+)+$/i) && rname != '') {
          adrs.push([email, rname, inpts[2].checked]);
        }
        if (email != '' && rname == '') {
          YAHOO.util.Dom.addClass(inpts[0], 'invalid');
        } else {
          YAHOO.util.Dom.removeClass(inpts[0], 'invalid');
        }
      }
      ManageSiteEmailInvitationsModule.vars.addresses = adrs;
    }
  },
  init: function (): void {
    ManageSiteEmailInvitationsModule.listeners.moreRecipients();
    ManageSiteEmailInvitationsModule.listeners.moreRecipients();
    ManageSiteEmailInvitationsModule.listeners.moreRecipients();
  }
};

ManageSiteEmailInvitationsModule.init();
