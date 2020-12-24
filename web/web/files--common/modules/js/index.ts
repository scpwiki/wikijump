import { ManageSiteEmailListsModule } from "./managesite/elists/ManageSiteEmailListsModule";
import { ManageSiteCloneModule } from "./managesite/ManageSiteCloneModule";
import { ManageSiteForumSettingsModule } from "./managesite/ManageSiteForumSettingsModule";
import { ManageSiteDomainModule } from "./managesite/ManageSiteDomainModule";
import { ManageSiteNotificationsModule } from "./managesite/ManageSiteNotificationsModule";
import { ManageSiteMembersApplicationsModule } from "./managesite/ManageSiteMembersApplicationsModule";
import { ManageSiteMembersModule } from "./managesite/ManageSiteMembersModule";
import { ManageSiteBackupModule } from "./managesite/backup/ManageSiteBackupModule";
import { ManageSiteAdminsModule } from "./managesite/ManageSiteAdminsModule";
import { ManageSitePermissionsModule } from "./managesite/ManageSitePermissionsModule";
import { ManageSiteMembersInviteModule } from "./managesite/ManageSiteMembersInviteModule";
import { ManageSiteInvitationsHistoryModule } from "./managesite/ManageSiteInvitationsHistoryModule";
import { ManageSiteForumPermissionsModule } from "./managesite/ManageSiteForumPermissionsModule";
import { ManageSitePrivateSettingsModule } from "./managesite/ManageSitePrivateSettingsModule";
import { ManageSiteMembersListModule } from "./managesite/ManageSiteMembersListModule";
import { ManageSitePerPageDiscussionModule } from "./managesite/ManageSitePerPageDiscussionModule";
import { ManageSiteEmailInvitationsModule } from "./managesite/ManageSiteEmailInvitationsModule";
import { ManageSiteSecureAccessModule } from "./managesite/ManageSiteSecureAccessModule";
import { ManageSiteLetUsersInviteModule } from "./managesite/ManageSiteLetUsersInviteModule";
import { ManageSiteRenameModule } from "./managesite/ManageSiteRenameModule";
import { ManageSiteDeleteModule } from "./managesite/ManageSiteDeleteModule";
import { ManageSiteNavigationModule } from "./managesite/ManageSiteNavigationModule";
import { ManageSiteCustomThemesModule } from "./managesite/ManageSiteCustomThemesModule";
import { ManageSiteForumLayoutModule } from "./managesite/ManageSiteForumLayoutModule";
import { ManageSitePageAbuseModule } from "./managesite/abuse/ManageSitePageAbuseModule";
import { ManageSiteAnonymousAbuseModule } from "./managesite/abuse/ManageSiteAnonymousAbuseModule";
import { ManageSiteUserAbuseModule } from "./managesite/abuse/ManageSiteUserAbuseModule";
import { ManageSiteUserBlocksModule } from "./managesite/blocks/ManageSiteUserBlocksModule";
import { ManageSiteIpBlocksModule } from "./managesite/blocks/ManageSiteIpBlocksModule";
import { ManageSiteModeratorsModule } from "./managesite/ManageSiteModeratorsModule";
import { ManageSiteAppearanceModule } from "./managesite/ManageSiteAppearanceModule";
import { ManageSiteGeneralModule } from "./managesite/ManageSiteGeneralModule";
import { ManageSiteTemplatesModule } from "./managesite/ManageSiteTemplatesModule";
import { ManageSiteOpenIDModule } from "./managesite/ManageSiteOpenIDModule";
import { ManageSitePageRateSettingsModule } from "./managesite/pagerate/ManageSitePageRateSettingsModule";
import { ManageSiteLicenseModule } from "./managesite/ManageSiteLicenseModule";
import { ManageSiteModule } from "./managesite/ManageSiteModule";
import { ForumCommentsModule } from "./forum/ForumCommentsModule";
import { ForumViewThreadModule } from "./forum/ForumViewThreadModule";
import { ForumRecentPostsModule } from "./forum/ForumRecentPostsModule";
import { ForumEditThreadStickinessModule } from "./forum/sub/ForumEditThreadStickinessModule";
import { ForumEditThreadBlockModule } from "./forum/sub/ForumEditThreadBlockModule";
import { ForumNewPostFormModule } from "./forum/sub/ForumNewPostFormModule";
import { ForumEditPostFormModule } from "./forum/sub/ForumEditPostFormModule";
import { ForumThreadMoveModule } from "./forum/sub/ForumThreadMoveModule";
import { ForumDeletePostModule } from "./forum/sub/ForumDeletePostModule";
import { ForumEditThreadMetaModule } from "./forum/sub/ForumEditThreadMetaModule";
import { ForumNewThreadModule } from "./forum/ForumNewThreadModule";
import { AccountApplicationsModule } from "./account/membership/AccountApplicationsModule";
import { AccountAdminOfModule } from "./account/membership/AccountAdminOfModule";
import { AccountWikiNewslettersModule } from "./account/membership/AccountWikiNewslettersModule";
import { AccountDeletedSitesModule } from "./account/membership/AccountDeletedSitesModule";
import { AccountModeratorOfModule } from "./account/membership/AccountModeratorOfModule";
import { AccountInvitationsModule } from "./account/membership/AccountInvitationsModule";
import { AccountMemberOfModule } from "./account/membership/AccountMemberOfModule";
import { AccountMessagesModule } from "./account/AccountMessagesModule";
import { AccountContactsModule } from "./account/contacts/AccountContactsModule";
import { ASPasswordModule } from "./account/settings/ASPasswordModule";
import { ASLanguageModule } from "./account/settings/ASLanguageModule";
import { ASNotificationsModule } from "./account/settings/ASNotificationsModule";
import { ASMessagesModule } from "./account/settings/ASMessagesModule";
import { ASBlockedModule } from "./account/settings/ASBlockedModule";
import { ASInvitationsModule } from "./account/settings/ASInvitationsModule";
import { ASEmailModule } from "./account/settings/ASEmailModule";
import { AccountNotificationsModule } from "./account/AccountNotificationsModule";
import { ChangeScreenNameModule } from "./account/profile/ChangeScreenNameModule";
import { APAboutModule } from "./account/profile/APAboutModule";
import { APAvatarModule } from "./account/profile/APAvatarModule";
import { PMComposeModule } from "./account/pm/PMComposeModule";
import { PMInboxModule } from "./account/pm/PMInboxModule";
import { PMDraftsModule } from "./account/pm/PMDraftsModule";
import { PMSentModule } from "./account/pm/PMSentModule";
import { AccountModule } from "./account/AccountModule";
import { AWForumModule } from "./account/watch/AWForumModule";
import { AWChangesModule } from "./account/watch/AWChangesModule";

export const modules = {
  /* Account */
  AccountModule,
  AccountMessagesModule,
  AccountApplicationsModule,
  AccountAdminOfModule,
  AccountWikiNewslettersModule,
  AccountDeletedSitesModule,
  AccountModeratorOfModule,
  AccountInvitationsModule,
  AccountMemberOfModule,
  AccountContactsModule,
  ASPasswordModule,
  ASLanguageModule,
  ASNotificationsModule,
  ASMessagesModule,
  ASBlockedModule,
  ASInvitationsModule,
  ASEmailModule,
  AccountNotificationsModule,
  ChangeScreenNameModule,
  APAboutModule,
  APAvatarModule,
  PMComposeModule,
  PMInboxModule,
  PMDraftsModule,
  PMSentModule,
  AWForumModule,
  AWChangesModule,

  /* Forum */
  ForumCommentsModule,
  ForumViewThreadModule,
  ForumRecentPostsModule,
  ForumEditThreadStickinessModule,
  ForumEditThreadBlockModule,
  ForumNewPostFormModule,
  ForumEditPostFormModule,
  ForumThreadMoveModule,
  ForumDeletePostModule,
  ForumEditThreadMetaModule,
  ForumNewThreadModule,

  /* Site Manager */
  ManageSiteModule,
  ManageSiteEmailListsModule,
  ManageSiteCloneModule,
  ManageSiteForumSettingsModule,
  ManageSiteDomainModule,
  ManageSiteNotificationsModule,
  ManageSiteMembersApplicationsModule,
  ManageSiteMembersModule,
  ManageSiteBackupModule,
  ManageSiteAdminsModule,
  ManageSitePermissionsModule,
  ManageSiteMembersInviteModule,
  ManageSiteInvitationsHistoryModule,
  ManageSiteForumPermissionsModule,
  ManageSitePrivateSettingsModule,
  ManageSiteMembersListModule,
  ManageSitePerPageDiscussionModule,
  ManageSiteEmailInvitationsModule,
  ManageSiteSecureAccessModule,
  ManageSiteLetUsersInviteModule,
  ManageSiteRenameModule,
  ManageSiteDeleteModule,
  ManageSiteNavigationModule,
  ManageSiteCustomThemesModule,
  ManageSiteForumLayoutModule,
  ManageSitePageAbuseModule,
  ManageSiteAnonymousAbuseModule,
  ManageSiteUserAbuseModule,
  ManageSiteUserBlocksModule,
  ManageSiteIpBlocksModule,
  ManageSiteModeratorsModule,
  ManageSiteAppearanceModule,
  ManageSiteGeneralModule,
  ManageSiteTemplatesModule,
  ManageSiteOpenIDModule,
  ManageSitePageRateSettingsModule,
  ManageSiteLicenseModule,
};
