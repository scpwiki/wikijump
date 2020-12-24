import Wikijump from "@/javascript/Wikijump";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
declare const YAHOO: any;
declare type YahooResponse = any;
declare const fx: any;

// Set in templates/modules/managesite/ManageSiteModule.tpl
declare const smStartPage: keyof typeof modulesMapping | undefined;

export type Category = {
  category_id: number;
  name: string;
  theme_id: number;
  variant_theme_id: number | null;
  theme_default: boolean;
  theme_external_url: string;
  template_id: number | null;
  nav_default: boolean;
  top_bar_page_name: string;
  side_bar_page_name: string;
  license_default: boolean;
  license_id: number;
  license_other: string;
  per_page_discussion: boolean | null;
  permissions: string | null;
  permissions_default: boolean;
  rating: string;
};

const modulesMapping = {
  'sm-welcome': "managesite/ManageSiteWelcomeModule",
  'sm-general': "managesite/ManageSiteGeneralModule",
  'sm-domain': "managesite/ManageSiteDomainModule",
  'sm-appearance': "managesite/ManageSiteAppearanceModule",
  'sm-customthemes': "managesite/ManageSiteCustomThemesModule",
  'sm-license': "managesite/ManageSiteLicenseModule",
  'sm-permissions': "managesite/ManageSitePermissionsModule",
  'sm-private': "managesite/ManageSitePrivateSettingsModule",
  'sm-members': "managesite/ManageSiteMembersModule",
  'sm-admins': "managesite/ManageSiteAdminsModule",
  'sm-moderators': "managesite/ManageSiteModeratorsModule",
  'sm-admins-invite': "managesite/ManageSiteAdminsInviteModule",
  'sm-files': "files/manager/FileManagerModule",
  'sm-navigation': "managesite/ManageSiteNavigationModule",
  'sm-ma': "managesite/ManageSiteMembersApplicationsModule",
  'sm-members-list': "managesite/ManageSiteMembersListModule",
  'sm-members-invite': "managesite/ManageSiteMembersInviteModule",
  'sm-email-invitations': "managesite/ManageSiteEmailInvitationsModule",
  'sm-invitations-history': "managesite/ManageSiteInvitationsHistoryModule",
  'sm-forum-settings': "managesite/ManageSiteForumSettingsModule",
  'sm-forum-layout': "managesite/ManageSiteForumLayoutModule",
  'sm-forum-perm': "managesite/ManageSiteForumPermissionsModule",
  'sm-templates': "managesite/ManageSiteTemplatesModule",
  'sm-forum-perpage': "managesite/ManageSitePerPageDiscussionModule",
  'sm-forum-recent': "managesite/ManageSiteForumRecentModule",
  'sm-recent-changes': "managesite/ManageSiteRecentModule",
  'sm-user-blocks': "managesite/blocks/ManageSiteUserBlocksModule",
  'sm-ip-blocks': "managesite/blocks/ManageSiteIpBlocksModule",
  'sm-pagerate': "managesite/pagerate/ManageSitePageRateSettingsModule",
  'sm-abuse-page': "managesite/abuse/ManageSitePageAbuseModule",
  'sm-abuse-user': "managesite/abuse/ManageSiteUserAbuseModule",
  'sm-abuse-anonymous': "managesite/abuse/ManageSiteAnonymousAbuseModule",
  'sm-notifications': "managesite/ManageSiteNotificationsModule",
  'sm-backup': "managesite/backup/ManageSiteBackupModule",
  'sm-ssl': "managesite/ManageSiteSecureAccessModule",
  'sm-openid': "managesite/ManageSiteOpenIDModule",
  'sm-users-email-invitations': "managesite/ManageSiteLetUsersInviteModule",
  'sm-renamesite': "managesite/ManageSiteRenameModule",
  'sm-deletesite': "managesite/ManageSiteDeleteModule",
  'sm-email-lists': "managesite/elists/ManageSiteEmailListsModule",
  'sm-clonesite': "managesite/ManageSiteCloneModule"
};

export const ManageSiteModule = {
  vars: {
    currentId: null as null | keyof typeof modulesMapping,
    // Set by callbacks.menuClick
    categories: null as null | Category[]
  },
  listeners: {
    clickMenu: function (event: Event): void {
      let target = YAHOO.util.Event.getTarget(event);
      const id = <keyof typeof modulesMapping>target.id;
      target = target.parentNode;
      const list = target.getElementsByTagName("ul").item(0);
      if (!list) {
        // means this is the link somewhere... at least should be.
        ManageSiteModule.utils.loadModule(id);
      } else {
        if (target.tagName.toLowerCase() != 'li') { return; }
        // toggle "selected" class
        if (YAHOO.util.Dom.hasClass(target, "selected")) {
          const eff = new fx.Opacity(list, { duration: 200 });
          eff.custom(1, 0);
          setTimeout(() => YAHOO.util.Dom.removeClass(target, "selected"), 200);
        } else {
          YAHOO.util.Dom.addClass(target, "selected");
          const eff = new fx.Opacity(list, { duration: 200 });
          eff.setOpacity(0);
          eff.custom(0, 1);
        }
      }
    }
  },
  callbacks: {
    menuClick: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }

      OZONE.utils.setInnerHTMLContent("sm-action-area", response.body);
      OZONE.utils.formatDates("sm-action-area");
      if (response.categories != null) {
        ManageSiteModule.vars.categories = response.categories;
      }
    }
  },
  utils: {
    getCategoryById: function (categoryId: number): Category {
      const categories = ManageSiteModule.vars.categories;
      if (categories == null) {
        throw new Error();
      }
      // XXX This assumes the category always exists
      return categories.find(category => category.category_id === categoryId)!;
    },
    getCategoryByName: function (name: string): Category {
      const categories = ManageSiteModule.vars.categories;
      if (categories == null) {
        throw new Error();
      }
      // XXX This assumes the category always exists
      return categories.find(category => category.name === name)!;
    },
    loadModule: function (id: keyof typeof modulesMapping, options: RequestModuleParameters = {}): void {
      const module = modulesMapping[id];
      if (module) {
        // toggle current
        const currentId = ManageSiteModule.vars.currentId;
        if (currentId) { YAHOO.util.Dom.removeClass(currentId, "active"); }
        ManageSiteModule.vars.currentId = id;
        YAHOO.util.Dom.addClass(id, "active");
        OZONE.ajax.requestModule(module, options, ManageSiteModule.callbacks.menuClick, null, { clearRequestQueue: true });

        // make sure the parent is unfolded (if is a list)
        const parent = document.getElementById(id)!.parentElement!.parentElement!.parentElement!;

        const list = parent.getElementsByTagName("ul").item(0);

        if (list && parent.tagName.toLowerCase() == 'li' && !YAHOO.util.Dom.hasClass(parent, "selected")) {
          // unfold
          YAHOO.util.Dom.addClass(parent, "selected");
          const eff = new fx.Opacity(list, { duration: 200 });
          eff.setOpacity(0);
          eff.custom(0, 1);
        }
      }
    }
  },
  init: function (): void {
    YAHOO.util.Event.addListener("site-manager-menu", "click", ManageSiteModule.listeners.clickMenu);

    OZONE.dom.onDomReady(function (): void {
      const startPage = smStartPage ?? "sm-welcome";
      ManageSiteModule.utils.loadModule(startPage);
    }, "dummy-ondomready-block");
  }
};

ManageSiteModule.init();
