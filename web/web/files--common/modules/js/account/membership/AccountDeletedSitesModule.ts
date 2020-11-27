import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare const YAHOO: any;
declare type YahooResponse = any;
declare const fx: any;

// Defined in WikiLayout.tpl
declare const HTTP_SCHEMA: string;
declare const URL_DOMAIN: string;

// As defined by php/modules/account/membership/AccountDeletedSitesModule.php,
// which is as defined by php/db/base/DB/SiteBase.php
type SitesData = {
  [sideId: number]: {
    name: string;
    unix_name: string;
  };
};

export const AccountDeletedSitesModule = {
  vars: {
    sitesData: null as null | SitesData,
    siteId: null as null | number,
  },
  init: function (): void {
    AccountDeletedSitesModule.vars.sitesData = JSON.parse(
      OZONE.utils.unescapeHtml(
        // This element is in:
        // templates/modules/account/membership/AccountDeletedSitesModule.tpl
        document.getElementById("as-restore-site-data")!.innerHTML
      )
    );
  },
  listeners: {
    clickRestore: function (_event: Event | null, siteId: number): void {
      const sitesData = AccountDeletedSitesModule.vars.sitesData;
      document.getElementById("as-restore-site-name")!.innerHTML = sitesData![siteId].name;
      (<HTMLInputElement>document.getElementById("as-restore-site-unixname")!).value = sitesData![siteId].unix_name;
      document.getElementById("as-restore-site-box")!.style.display = 'block';
      AccountDeletedSitesModule.vars.siteId = siteId;
      OZONE.visuals.scrollTo("as-restore-site-box");
    },
    restore: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        siteId: AccountDeletedSitesModule.vars.siteId,
        unixName: (<HTMLInputElement>document.getElementById("as-restore-site-unixname")!).value,
        action: 'AccountMembershipAction',
        event: 'restoreSite',
      };
      OZONE.ajax.requestModule(null, params, AccountDeletedSitesModule.callbacks.restore);
      const w = new OZONE.dialogs.WaitBox();
      w.content = "Restoring the site...";
      w.show();
    }
  },
  callbacks: {
    restore: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const w = new OZONE.dialogs.SuccessBox();
      w.content = "The site has been restored.";
      w.show();
      // Navigate to the new site after a half-second
      setTimeout(() => window.location.href=`${HTTP_SCHEMA}://${response.unixName}.${URL_DOMAIN}`, 500);
    }
  }
};

AccountDeletedSitesModule.init();
