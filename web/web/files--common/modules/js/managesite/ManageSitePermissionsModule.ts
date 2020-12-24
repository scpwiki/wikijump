import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
declare const YAHOO: any;
declare type YahooResponse = any;

const users = <const>[
  'a', // anonymous
  'r', // registered at Wikijump
  'm', // member of the site
  'o', // owner (creator) of the page
];
type UserCode = (typeof users)[number];

const permissions = <const>[
  // 'v', // view page
  'e', // edit page
  'c', // create new pages
  'm', // move pages
  'd', // delete pages
  'a', // attach files
  'r', // rename files
  'z', // replace/move/delete files
  'o', // show page options to...
];
type PermissionsCode = (typeof permissions)[number];

/* Sample permissions string: v:arm;e:;c:;m:;d:;a:;r:;z:;o: */

export const ManageSitePermissionsModule = {
  listeners: {
    categoryChange: function (_event?: Event | null): void {
      // update permissions info
      const categoryId = parseInt((<HTMLSelectElement>document.getElementById("sm-perms-cats")!).value);
      const category = Wikijump.modules.ManageSiteModule.utils.getCategoryById(categoryId);
      // check if has a individual permissions
      if (category.name == "_default") {
        document.getElementById("sm-perms-noind")!.style.display = "none";
        document.getElementById("sm-perms-table")!.style.display = "";
      } else {
        document.getElementById("sm-perms-noind")!.style.display = "block";
        if (category.permissions_default == true) {
          (<HTMLInputElement>document.getElementById("sm-perms-noin")!).checked = true;
          document.getElementById("sm-perms-table")!.style.display = "none";
        } else {
          (<HTMLInputElement>document.getElementById("sm-perms-noin")!).checked = false;
          document.getElementById("sm-perms-table")!.style.display = "";
        }
      }
      let pstring = category.permissions;
      if ((pstring == null || pstring == '') && category.name != "_default") {
        // get a string from default category
        const defcat = Wikijump.modules.ManageSiteModule.utils.getCategoryByName("_default");
        pstring = defcat.permissions;
      }
      ManageSitePermissionsModule.utils.decodePermissions(pstring);
    },
    indClick: function (_event?: Event | null): void {
      const categoryId = parseInt((<HTMLSelectElement>document.getElementById("sm-perms-cats")!).value);
      const category = Wikijump.modules.ManageSiteModule.utils.getCategoryById(categoryId);

      if ((<HTMLInputElement>document.getElementById("sm-perms-noin")!).checked) {
        document.getElementById("sm-perms-table")!.style.display = "none";
        category.permissions_default = true;
      } else {
        document.getElementById("sm-perms-table")!.style.display = "";
        category.permissions_default = false;
      }
    },
    permissionChange: function (event: Event): void {
      // save changes to the array
      const categoryId = parseInt((<HTMLSelectElement>document.getElementById("sm-perms-cats")!).value);
      const category = Wikijump.modules.ManageSiteModule.utils.getCategoryById(categoryId);

      // fix permissions first (difficult?)
      const target = YAHOO.util.Event.getTarget(event);
      ManageSitePermissionsModule.utils.fixPermissions(target.id);
      // encode permissions and save
      const pstring = ManageSitePermissionsModule.utils.encodePermissions();
      category.permissions = pstring;
    },
    cancel: function (_event?: Event | null): void {
      Wikijump.modules.ManageSiteModule.utils.loadModule('sm-welcome');
    },
    save: function (_event?: Event | null): void {
      // ok, do it the easy way: serialize categories using the JSON method
      const categories = Wikijump.modules.ManageSiteModule.vars.categories;
      const serialized = JSON.stringify(categories);
      const params = {
        categories: serialized,
        action: "ManageSiteAction",
        event: "savePermissions"
      };
      OZONE.ajax.requestModule("Empty", params, ManageSitePermissionsModule.callbacks.save);
      const w = new OZONE.dialogs.WaitBox();
      w.content = "Saving permissions...";
      w.show();
    }
  },
  callbacks: {
    cancel: function (response: YahooResponse): void {
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
      const farray = OZONE.utils.formToArray("sm-perms-form");
      // now traverse the form...
      let i, j;
      let out = ''; // output
      let tag;
      for (i = 0; i < permissions.length; i++) {
        if (i > 0) { out += ";"; }
        out += permissions[i] + ':';
        for (j = 0; j < users.length; j++) {
          tag = permissions[i] + '-' + users[j];
          // find a checkbox and check value
          if (farray[tag] == 'on') {
            out += users[j];
          }
        }
      }
      return out;
    },
    decodePermissions: function (pstring: string): void {
      // clear the table
      let i, j;
      for (i = 0; i < permissions.length; i++) {
        for (j = 0; j < users.length; j++) {
          const tag = `sm-${permissions[i]}-${users[j]}`;
          const el = (<HTMLInputElement>document.getElementById(tag)!);
          if (el) {
            el.checked = false;
          }
        }
      }
      if (pstring != null && pstring !== '') {
        const activs = pstring.split(';');
        for (i = 0; i < activs.length; i++) {
          const activs2 = <[PermissionsCode, string]>activs[i].split(':');
          const activName = activs2[0];
          const activPerms = activs2[1];
          for (j = 0; j < activPerms.length; j++) {
            const activUser = <UserCode>activPerms.charAt(j);
            // now set the checkbox
            const tag = `sm-${activName}-${activUser}`;
            const el = (<HTMLInputElement>document.getElementById(tag)!);
            if (el) {
              el.checked = true;
            }
          }
        }
      }
    },
    fixPermissions: function (id: string): void {
      // an ugly way...
      const el = (<HTMLInputElement>document.getElementById(id)!);
      // id is expected to start with 'sm-'
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
    YAHOO.util.Event.addListener("sm-perms-cats", "change", ManageSitePermissionsModule.listeners.categoryChange);
    YAHOO.util.Event.addListener("sm-perms-noind", "click", ManageSitePermissionsModule.listeners.indClick);

    YAHOO.util.Event.addListener("sm-perms-form", "click", ManageSitePermissionsModule.listeners.permissionChange);
    // do it the other way...

    YAHOO.util.Event.addListener("sm-perms-cancel", "click", ManageSitePermissionsModule.listeners.cancel);
    YAHOO.util.Event.addListener("sm-perms-save", "click", ManageSitePermissionsModule.listeners.save);
    // init categories info
    ManageSitePermissionsModule.listeners.categoryChange();
  }
};

ManageSitePermissionsModule.init();
