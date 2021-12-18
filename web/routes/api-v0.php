<?php
declare(strict_types=1);

use Illuminate\Http\Request;
use Illuminate\Support\Facades\App;
use Illuminate\Support\Facades\Http;
use Illuminate\Support\Facades\Log;
use Illuminate\Support\Facades\Route;
use Wikijump\Http\Controllers\AccountController;
use Wikijump\Http\Controllers\AuthController;

// -- QUERY
// TODO: queryRequest

// -- UTIL
// TODO: utilResolveID

// -- AUTH
Route::post('/auth/login', [AuthController::class, 'login']);
Route::post('/auth/confirm', [AuthController::class, 'confirm']);
Route::delete('/auth/logout', [AuthController::class, 'logout']);
Route::post('/auth/check', [AuthController::class, 'check']);
Route::post('/auth/refresh', [AuthController::class, 'refresh']);

// -- ACCOUNT
// these can be implemented!
Route::post('/account/register', [AccountController::class, 'register']);
Route::post('/account/send-verification-email', [
    AccountController::class,
    'sendVerificationEmail',
]);
// TODO: accountRequestDeletion
Route::post('/account/start-recovery', [AccountController::class, 'startRecovery']);
// TODO: accountGetEmail
// TODO: accountUpdateEmail
// TODO: accountUpdatePassword
// TODO: accountGetUsername
// TODO: accountUpdateUsername
// TODO: accountGetSettings
// TODO: accountUpdateSettings

// -- NOTIFICATION
// these can be implemented!
// TODO: notificationGet
// TODO: notificationDismissAll

// -- USER
// these can be implemented!
// TODO: userClientGet
// TODO: userClientUpdateProfile
// TODO: userClientGetAvatar
// TODO: userClientSetAvatar
// TODO: userClientRemoveAvatar
// TODO: userClientGetBlocked
// TODO: userGet
// TODO: userResetProfile
// TODO: userGetAvatar
// TODO: userRemoveAvatar
// TODO: userGetBlocked
// TODO: userUpdateBlocked

// -- MEMBERSHIP
// TODO: membershipGetList
// TODO: membershipGetApplications
// TODO: membershipGetInvites
// TODO: membershipSiteGet
// TODO: membershipSiteApply
// TODO: membershipSiteLeave
// TODO: membershipUserGetList
// TODO: membershipUserSiteGet
// TODO: membershipUserGetRole
// TODO: membershipUserSetRole
// TODO: membershipUserInvite

// -- PAGE
// TODO: pageCreate
// TODO: pageGet
// TODO: pageUpdate
// TODO: pageDelete
// TODO: pageRestore
// TODO: pageRename

// -- REVISION
// TODO: revisionPageGetHistory
// TODO: revisionGet
// TODO: revisionUpdateMetadata
// TODO: revisionResetToRevision

// -- TAG
// TODO: tagPageGet
// TODO: tagPageUpdate

// -- VOTE
// TODO: votePageGetScore
// TODO: votePageGetVoters
// TODO: votePageGet
// TODO: votePageUpdateVote
// TODO: votePageRemoveVote

// -- FILE
// TODO: filePageGetMetadata
// TODO: filePageAdd
// TODO: fileSiteGetMetadata
// TODO: fileSiteAdd
// TODO: fileGet
// TODO: fileDelete
// TODO: fileGetSiteMetadata
// TODO: fileGetMetadata

// -- REPORT
// TODO: reportUserGet
// TODO: reportUserSend
// TODO: reportPageGet
// TODO: reportPageSend
// TODO: reportGet

// -- ABUSE
// TODO: abuseSiteGet
// TODO: abuseSiteSend
// TODO: abuseUserGet
// TODO: abuseUserSend
// TODO: abusePageGet
// TODO: abusePageSend
// TODO: abuseGet

// -- MESSAGES
// these can be implemented!
// TODO: messageGet
// TODO: messageUpdate
// TODO: messageDelete
// TODO: messageSend

// -- FORUM-MISC
// TODO: forumGet

// -- FORUM-GROUP
// TODO: forumGroupGetList
// TODO: forumGroupGet
// TODO: forumGroupUpdate
// TODO: forumGroupAddCategory
// TODO: forumGroupDelete
// TODO: forumGroupGetCategories

// -- FORUM-CATEGORY
// TODO: forumCategoryGetList
// TODO: forumCategoryGet
// TODO: forumCategoryUpdate
// TODO: forumCategoryAddThread
// TODO: forumCategoryDelete
// TODO: forumCategoryGetThreads

// -- FORUM-THREAD
// TODO: forumThreadGet
// TODO: forumThreadUpdate
// TODO: forumThreadAddPost
// TODO: forumThreadDelete
// TODO: forumThreadGetPosts

// -- FORUM-POST
// TODO: forumPostGet
// TODO: forumPostUpdate
// TODO: forumPostReply
// TODO: forumPostDelete
// TODO: forumPostGetReplies
// TODO: forumPostRevisionGetHistory
// TODO: forumPostRevisionGet
// TODO: forumPostRevisionUpdateMetadata
// TODO: forumPostResetToRevision

// -- MODERATION
// TODO: moderationKick
// TODO: moderationBanGetList
// TODO: moderationBanGet
// TODO: moderationBan
// TODO: moderationUnban

// -- CATEGORY
// TODO: categoryGetList
// TODO: categoryDefaultGet
// TODO: categoryDefaultPatch
// TODO: categoryGet
// TODO: categoryPatch

// -- SITE
// TODO: siteSettingsGet
// TODO: siteSettingsPatch
// TODO: siteApplicationGetList
// TODO: siteApplicationGet
// TODO: siteApplicationAccept
// TODO: siteApplicationReject
// TODO: siteBackupGet
// TODO: siteCreate
// TODO: siteRequestDeletion
// TODO: siteNotificationGet
// TODO: siteNotificationDismissAll
// TODO: siteNewsletterSend
// TODO: siteTransfer

// Fallback to Mockoon server, if it's up
if (App::environment('local')) {
    Route::any('/{path}', function (Request $request, $path) {
        try {
            $verb = $request->method();
            $res = Http::send($verb, "http://host.docker.internal:3500/api--v0/$path", [
                'headers' => [
                    'Accept' => 'application/json',
                    'Content-Type' => $request->header('Content-Type'),
                ],
                'query' => $request->query->all(),
                'body' => $request->getContent(),
            ]);

            Log::debug("Proxied unimplemented API path ('$path') to Mockoon");

            return response($res->body(), $res->status(), $res->headers());
        } catch (Exception $err) {
            // server probably isn't up, as otherwise we would've gotten a 404
            return response(null, 404);
        }
    })->where('path', '.*');
}
