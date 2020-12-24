import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
import { Wikirequest } from "wikirequest";
declare const WIKIREQUEST: Wikirequest;
declare const YAHOO: any;
declare type YahooResponse = any;

export const ManageSitePrivateSettingsModule = {
  listeners: {
    save: function (_event?: Event | null): void {
      // get "viewers" list
      const sr = document.getElementById("viewers-list-div")!;
      const ents = sr.getElementsByTagName('div');
      const uss = [];
      for (let i = 0; i < ents.length; i++) {
        const userId = ents[i].id.replace(/.*?([0-9]+)$/, "$1");
        uss.push(userId);
      }
      const params: RequestModuleParameters = {
        ...OZONE.utils.formToArray("sm-private-form"),
        viewers: uss.join(','),
        action: "ManageSiteAction",
        event: "savePrivateSettings",
      };

      OZONE.ajax.requestModule(null, params, ManageSitePrivateSettingsModule.callbacks.save);
      const w = new OZONE.dialogs.WaitBox();
      w.content = "Saving changes...";
      w.show();
    }
  },
  callbacks: {
    save: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const w = new OZONE.dialogs.SuccessBox();
      w.content = "Changes saved";
      w.show();
    }
  },
  utils: {
    addViewer: function (userId: number, userName: string): void {
      const cont = document.getElementById("viewers-list-div")!;
      const vid = "viewer-entry-" + userId;

      if (!document.getElementById(vid)!) {
        // add user
        const di = document.createElement('div');
        di.id = vid;
        di.innerHTML = userName;
        cont.appendChild(di);
        ManageSitePrivateSettingsModule.utils.updateViewers();
      }
      (<HTMLInputElement>document.getElementById("user-lookup")!).value = '';
    },
    removeUser: function (userId: number): void {
      const cont = document.getElementById("viewers-list-div")!;
      const vid = "viewer-entry-" + userId;

      if (document.getElementById(vid)!) {
        cont.removeChild(document.getElementById(vid)!);
        ManageSitePrivateSettingsModule.utils.updateViewers();
      }
    },
    updateViewers: function (): void {
      const dcont = document.getElementById("extra-viewers-display-list")!;
      const sr = document.getElementById("viewers-list-div")!;
      const ents = sr.getElementsByTagName('div');
      const uss = [];
      for (let i = 0; i < ents.length; i++) {
        const userId = parseInt(ents[i].id.replace(/.*?([0-9]+)$/, "$1"));
        let str = Wikijump.render.printuser(userId, ents[i].innerHTML, true);
        str += '(<a href="javascript:;" title="remove from the list" onclick="ManageSitePrivateSettingsModule.utils.removeUser(' + userId + ')">x</a>)';
        uss.push(str);
      }
      if (uss.length == 0) {
        dcont.innerHTML = 'No extra access granted.';
      } else {
        dcont.innerHTML = uss.join(', ');
      }
    }
  },
  init: function (): void {
    // attach the autocomplete thing
    const myDataSource = new YAHOO.widget.DS_XHR("/quickmodule.php", ['pages', 'unix_name', 'title']);
    myDataSource.scriptQueryParam = "q";
    myDataSource.scriptQueryAppend = "s=" + WIKIREQUEST.info.siteId + "&module=PageLookupQModule";

    const myAutoComp = new YAHOO.widget.AutoComplete("sm-private-land", "sm-private-land-list", myDataSource);
    // @ts-expect-error Autocomp
    myAutoComp.formatResult = function (aResultItem, _sQuery): string {
      const title = aResultItem[1];
      const unixName = aResultItem[0];
      if (unixName != null) {
        return '<div style="font-size: 100%">' + unixName + '</div><div style="font-size: 80%;">(' + title + ')</div>';
      } else {
        return "";
      }
    };

    myAutoComp.autoHighlight = false;
    myAutoComp.minQueryLength = 2;
    myAutoComp.queryDelay = 0.5;

    // init autocomplete now
    const dataSource = new YAHOO.widget.DS_XHR("/quickmodule.php", ['users', 'name', 'user_id']);
    dataSource.scriptQueryParam = "q";
    dataSource.scriptQueryAppend = "&module=UserLookupQModule";

    const autoComp = new YAHOO.widget.AutoComplete('user-lookup', 'user-lookup-list', dataSource);

    autoComp.minQueryLength = 2;
    autoComp.queryDelay = 0.5;
    autoComp.forceSelection = true;
    // @ts-expect-error Autocomp
    autoComp.itemSelectEvent.subscribe(function (_sType, args): void {
      const userId = args[1].getElementsByTagName('div').item(0).id.replace(/.*?([0-9]+)$/, "$1");
      const userName = args[1].getElementsByTagName('div').item(0).innerHTML;
      ManageSitePrivateSettingsModule.utils.addViewer(userId, userName);
    });

    // @ts-expect-error Autocomp
    autoComp.formatResult = function (aResultItem, _sQuery): string {
      const name = aResultItem[0];
      const userId = aResultItem[1];
      if (name != null) {
        return '<div id="user-autocomplete-' + userId + '">' + name + '</div>';
      } else {
        return "";
      }
    };

    ManageSitePrivateSettingsModule.utils.updateViewers();
  }
};

ManageSitePrivateSettingsModule.init();
