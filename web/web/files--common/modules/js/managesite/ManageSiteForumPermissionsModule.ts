import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare const YAHOO: any;
declare type YahooResponse = any;

// XXX categoryId should be a number, but code asserts it can be ''

const users = <const>[
  'a', // anonymous
  'r', // registered at Wikijump
  'm', // member of the site
  'o', // author of the post
];
type UserCode = (typeof users)[number];

const permissions = <const>[
  't', // start new threads
  'p', // add new posts
  'e', // edit posts/threads (!!!)
  's', // split - create new threads from existing posts
];
type PermissionsCode = (typeof permissions)[number];

export const ManageSiteForumPermissionsModule = {
  vars: {
    defaultPermissions: "" // Set by init
  },
  listeners: {
    categoryChange: function (_event?: Event | null): void {
      // update permissions info
      const categoryId = (<HTMLSelectElement>document.getElementById("sm-perms-cats")!).value;
      let pstring;
      if (categoryId == '') {
        // default permissions
        document.getElementById("sm-perms-noind")!.style.display = "none";
        document.getElementById("sm-perms-table")!.style.display = "";
        pstring = ManageSiteForumPermissionsModule.vars.defaultPermissions;
      } else {
        document.getElementById("sm-perms-noind")!.style.display = "block";
        const category = Wikijump.modules.ManageSiteModule.utils.getCategoryById(parseInt(categoryId));
        if (category.permissions == null) {
          (<HTMLInputElement>document.getElementById("sm-perms-noin")!).checked = true;
          document.getElementById("sm-perms-table")!.style.display = "none";
          pstring = ManageSiteForumPermissionsModule.vars.defaultPermissions;
        } else {
          (<HTMLInputElement>document.getElementById("sm-perms-noin")!).checked = false;
          document.getElementById("sm-perms-table")!.style.display = "block";
          pstring = category.permissions;
        }
      }

      ManageSiteForumPermissionsModule.utils.decodePermissions(pstring);
    },
    indClick: function (_event?: Event | null): void {
      const categoryId = (<HTMLSelectElement>document.getElementById("sm-perms-cats")!).value;
      if (categoryId == "") return; // should not be

      const category = Wikijump.modules.ManageSiteModule.utils.getCategoryById(parseInt(categoryId));

      if ((<HTMLInputElement>document.getElementById("sm-perms-noin")!).checked == true) {
        document.getElementById("sm-perms-table")!.style.display = "none";
        category.permissions = null;
      } else {
        document.getElementById("sm-perms-table")!.style.display = "";
        category.permissions = ManageSiteForumPermissionsModule.vars.defaultPermissions;
      }
    },
    permissionChange: function (event: Event): void {
      // fix permissions first (difficult?)
      const target = YAHOO.util.Event.getTarget(event);
      ManageSiteForumPermissionsModule.utils.fixPermissions(target.id);
      // encode permissions and save
      const pstring = ManageSiteForumPermissionsModule.utils.encodePermissions();
      // save changes to the array
      const categoryId = (<HTMLSelectElement>document.getElementById("sm-perms-cats")!).value;
      if (categoryId == '') {
        ManageSiteForumPermissionsModule.vars.defaultPermissions = pstring;
      } else {
        const category = Wikijump.modules.ManageSiteModule.utils.getCategoryById(parseInt(categoryId));
        category.permissions = pstring;
      }
    },
    cancel: function (_event?: Event | null): void {
      Wikijump.modules.ManageSiteModule.utils.loadModule('sm-welcome');
    },
    save: function (_event?: Event | null): void {
      // ok, do it the easy way: serialize categories using the JSON method
      const categories = Wikijump.modules.ManageSiteModule.vars.categories;
      const serialized = JSON.stringify(categories);
      const params: RequestModuleParameters = {
        categories: serialized,
        default_permissions: ManageSiteForumPermissionsModule.vars.defaultPermissions,
        action: "ManageSiteForumAction",
        event: "saveForumPermissions"
      };
      OZONE.ajax.requestModule("Empty", params, ManageSiteForumPermissionsModule.callbacks.save);
      const w = new OZONE.dialogs.WaitBox();
      w.content = "Saving permissions...";
      w.show();
    }
  },
  callbacks: {
    cancel: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      OZONE.utils.setInnerHTMLContent("site-manager", response.body);
    },
    save: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }

      const w = new OZONE.dialogs.SuccessBox();
      w.content = "Permissions have been saved.";
      w.show();
    }
  },
  utils: {
    encodePermissions: function (): string {
      // now traverse the form...
      let i, j;
      let out = ''; // output
      let tag;
      for (i = 0; i < permissions.length; i++) {
        if (i > 0) { out += ";"; }
        out += permissions[i] + ':';
        for (j = 0; j < users.length; j++) {
          // find a checkbox and check value
          tag = `sm-${permissions[i]}-${users[j]}`;
          if (document.getElementById(tag)! && (<HTMLInputElement>document.getElementById(tag)!).checked == true) {
            out += users[j];
          }
        }
      }
      return out;
    },
    decodePermissions: function (pstring: string): void {
      let activName;
      let activPerms;
      let activUser;
      let tag;
      let el;

      // clear the table
      let i, j;
      for (i = 0; i < permissions.length; i++) {
        for (j = 0; j < users.length; j++) {
          tag = 'sm-' + permissions[i] + '-' + users[j];
          el = (<HTMLInputElement>document.getElementById(tag)!);
          if (el) {
            el.checked = false;
          }
        }
      }
      if (pstring != null && pstring != '') {
        const activs = pstring.split(';');
        for (i = 0; i < activs.length; i++) {
          const activs2 = activs[i].split(':');
          activName = activs2[0];
          activPerms = activs2[1];
          for (j = 0; j < activPerms.length; j++) {
            activUser = activPerms.charAt(j);
            // now set the checkbox
            tag = 'sm-' + activName + '-' + activUser;
            el = (<HTMLInputElement>document.getElementById(tag)!);
            if (el) { el.checked = true; }
          }
        }
      }
    },
    fixPermissions: function (id: string): void {
      // an ugly way...
      const el = (<HTMLInputElement>document.getElementById(id)!);

      const tsplit = id.split("-");
      const activ = <PermissionsCode>tsplit[1];
      const user = <UserCode>tsplit[2];
      let charray: UserCode[] = [];
      if (el.checked == true) {
        switch (user) {
          case "a":
            charray = ['r', 'm'];
          break;
          case 'r':
            charray = ['m'];
          break;
          case 'm':
            charray = [];
          break;
        }
        for (let i = 0; i < charray.length; i++) {
          const tag = `sm-${activ}-${charray[i]}`;
          const el2 = (<HTMLInputElement>document.getElementById(tag)!);
          if (el2) {
            el2.checked = true;
          }
        }
      }
      if (el.checked == false) {
        switch (user) {
          case "r":
            charray = ['a'];
          break;
          case 'm':
            charray = ['a', 'r'];
          break;
          case 'a':
            charray = [];
          break;
        }
        for (let i = 0; i < charray.length; i++) {
          const tag = `sm-${activ}-${charray[i]}`;
          const el2 = (<HTMLInputElement>document.getElementById(tag)!);
          if (el2) {
            el2.checked = true;
          }
        }
      }
    }
  },
  init: function (): void {
    YAHOO.util.Event.addListener("sm-perms-cats", "change", ManageSiteForumPermissionsModule.listeners.categoryChange);
    ManageSiteForumPermissionsModule.vars.defaultPermissions = (<HTMLInputElement>document.getElementById("default-forum-permissions")!).value;
    YAHOO.util.Event.addListener("sm-perms-noind", "click", ManageSiteForumPermissionsModule.listeners.indClick);
    YAHOO.util.Event.addListener("sm-perms-form", "click", ManageSiteForumPermissionsModule.listeners.permissionChange);
    YAHOO.util.Event.addListener("sm-perms-cancel", "click", ManageSiteForumPermissionsModule.listeners.cancel);
    YAHOO.util.Event.addListener("sm-perms-save", "click", ManageSiteForumPermissionsModule.listeners.save);

    ManageSiteForumPermissionsModule.listeners.categoryChange(null);
  }
};
ManageSiteForumPermissionsModule.init();
