import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
declare const YAHOO: any;
declare type YahooResponse = any;
declare const fx: any;

// Set in templates/modules/account/AccountModule.tpl
declare const accountStartPage: keyof typeof mapping;

const mapping = {
  'am-welcome': "account/AccountWelcomeModule",
  'am-messages': "account/AccountMessagesModule",
  'am-notifications': "account/AccountNotificationsModule",
  'am-contacts': "account/contacts/AccountContactsModule",
  'am-profile': "account/AccountProfileModule",
  'am-adminof': "account/membership/AccountAdminOfModule",
  'am-moderatorof': "account/membership/AccountModeratorOfModule",
  'am-memberof': "account/membership/AccountMemberOfModule",
  'am-invitations': "account/membership/AccountInvitationsModule",
  'am-applications': "account/membership/AccountApplicationsModule",
  'am-recentcontrib': "userinfo/UserChangesModule",
  'am-recentposts': "userinfo/UserRecentPostsModule",
  'am-stats': "account/AccountStatisticsModule",
  'am-settings': "account/AccountSettingsModule",
  'am-watched-changes': "account/watch/AWChangesModule",
  'am-watched-forum': "account/watch/AWForumModule",
  'am-watched-feed': "account/watch/AWFeedModule",
  'am-wiki-newsletters': "account/membership/AccountWikiNewslettersModule",
  'am-deletedsites': "account/membership/AccountDeletedSitesModule"
};

export const AccountModule = {
  vars: {
    currentId: null as null | keyof typeof mapping
  },
  mapping,
  listeners: {
    clickMenu: function (event: Event): void {
      let target = YAHOO.util.Event.getTarget(event);
      const id = target.id;
      target = target.parentNode;
      const list = target.getElementsByTagName("ul").item(0);
      if (!list) {
        // means this is the link somewhere... at least should be.
        AccountModule.utils.loadModule(id);
      } else {
        if (target.tagName.toLowerCase() != 'li') { return; }
        // toggle "selected" class
        if (YAHOO.util.Dom.hasClass(target, "selected")) {
          const eff = new fx.Opacity(list, { duration: 200 });
          eff.custom(1, 0);
          const tz = target;
          setTimeout(() => YAHOO.util.Dom.removeClass(tz,"selected"), 200);
        } else {
          YAHOO.util.Dom.addClass(target, "selected");
          const eff = new fx.Opacity(list, { duration: 200 });
          eff.setOpacity(0);
          eff.custom(0, 1);
        }
      }
    },
  },
  callbacks: {
    menuClick: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      document.getElementById("account-area")!.innerHTML = response.body;
      OZONE.utils.formatDates(document.getElementById("account-area")!);
    }
  },
  utils: {
    loadModule: function (id: keyof typeof mapping): void {
      const mm = AccountModule.mapping;
      const module = mm[id];
      if (module) {
        // toggle current
        const currentId = AccountModule.vars.currentId;
        if (currentId) { YAHOO.util.Dom.removeClass(currentId, "active"); }
        AccountModule.vars.currentId = id;
        YAHOO.util.Dom.addClass(id, "active");
        OZONE.ajax.requestModule(module, {}, AccountModule.callbacks.menuClick, null, { clearRequestQueue: true });

        // make sure the parent is unfolded (if is a list)
        const p = document.getElementById(id)!.parentElement!.parentElement!.parentElement!;

        const list = p.getElementsByTagName("ul").item(0);

        if (list && p.tagName.toLowerCase() == 'li' && !YAHOO.util.Dom.hasClass(p, "selected")) {
          // unfold
          YAHOO.util.Dom.addClass(p, "selected");
          const eff = new fx.Opacity(list, { duration: 200 });
          eff.setOpacity(0);
          eff.custom(0, 1);
        }
      }
    }
  },
  init: function (): void {
    YAHOO.util.Event.addListener("account-side", "click", AccountModule.listeners.clickMenu);


    OZONE.dom.onDomReady(function (): void {
      if (!document.getElementById("account-area")!) {
        return;
      }

      let startPage: keyof typeof mapping = "am-welcome";
      if (accountStartPage) {
        startPage = accountStartPage;
      }
      // on DOM complete!!!

      AccountModule.utils.loadModule(startPage);
    }, "dummy-ondomready-block");
  }
};

AccountModule.init();
