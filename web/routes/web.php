<?php

use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Facades\Route;
use Illuminate\Support\Facades\URL;
use Laravel\Fortify\Http\Controllers\AuthenticatedSessionController;
use Ozone\Framework\Ozone;
use Ozone\Framework\RunData;
use Wikidot\Utils\AjaxModuleWikiFlowController;
use Wikidot\Utils\GlobalProperties;
use Wikijump\Helpers\LegacyTools;
use Wikijump\Http\Controllers\OzoneController;
use Wikijump\Models\User;

use const Wikijump\Helpers\LegacyTools;

/**
 * We are instantiating a RunData object with every route request so we can pull
 * some information related to sessions. Hopefully this won't stay in for long.
 */
if( php_sapi_name() !== "cli" ) {
    Ozone::init();
    $runData = new RunData();
    $runData->init();
    Ozone:: setRunData($runData);
    $runData->handleSessionStart();
    if ($runData->getUserId() && Auth::guest()) {
        Auth::login(
            User::findorFail($runData->getUserId())
        );
    };
}



/*
|--------------------------------------------------------------------------
| Web Routes
|--------------------------------------------------------------------------
|
| Here is where you can register web routes for your application. These
| routes are loaded by the RouteServiceProvider within a group which
| contains the "web" middleware group. Now create something great!
|
*/
Route::get('welcome', function () {
    return view('welcome');
});

// TODO: remove these when we have a proper frontend, this is just for testing

Route::get('/editor--test', function () {
    return view('next.test.editor-test');
});

/**
 * Socialite route, null until I'm ready to begin work there.
 */
Route::prefix('social--providers')->group(function() {

    Route::get('/callback', function ($provider) {
        return app()
            ->call(
                'Wikijump\Http\Controllers\SocialiteController@callback',
                ['provider' => $provider]);
    })->name('socialite-callback');

});

/**
 * AJAX Handler, formerly ajax-module-connector.php
 */
Route::post('/ajax--handler', function() {
    $controller = new AjaxModuleWikiFlowController();
    $controller->process();
});

/**
 * Karma displayer.
 */
Route::get('/user--karma/{user}', function(User $user) {
    $karma = Cache::remember('karma_level__user_'.$user->id, 3600, function() use($user) {
        return $user->karma_level;
    });
    header('Content-type: image/png');
    header('Expires: ' . gmdate('D, d M Y H:i:s', time() + 3600) . ' GMT');
    header("Last-Modified: ".gmdate('D, d M Y H:i:s', time() ) . ' GMT');
    header('Cache-Control: max-age=3600, must-revalidate');
    readfile(WIKIJUMP_ROOT.'/web/files--common/theme/base/images/karma/karma_'.$karma.'.png');
});

/**
 * Avatar shortcut
 */
Route::get('/user--avatar/{user}', function (User $user) {
   return $user->avatar();
});

/**
 * This route will use Blade instead of Smarty for rendering.
 */
Route::get('/what-is-a-wiki', function() {
   $values = LegacyTools::generateScreenVars();
   return view('layouts.legacy', [
       'site' => $values['site'] ?? null,
       'pageNotExists' => $values['pageNotExists'] ?? null,
       'category' => $values['category'] ?? null,
       'theme' => $values['theme'] ?? null,
       'wikiPage' => ($values['wikiPage'] ?? null),
       'wikiPageName' => ($values['wikiPageName'] ?? null),
       'pageContent' => ($values['pageContent'] ?? null),
       'pageParameters' => ($values['pageParameters'] ?? null),
       'topBarContent' => $values['topBarContent'] ?? null,
       'sideBar1Content' => $values['sideBar1Content'] ?? null,
       'breadcrumbs' => $values['breadcrumbs'] ?? null,
       'tags' => $values['tags'] ?? null,
       'licenseHtml' => $values['licenseHtml'] ?? null,
       'HTTP_SCHEMA' => GlobalProperties::$HTTP_SCHEMA,
       'URL_DOMAIN' => GlobalProperties::$URL_DOMAIN,
       'URL_HOST' => GlobalProperties::$URL_HOST,
       'SERVICE_NAME' => GlobalProperties::$SERVICE_NAME,
       'usePrivateWikiScript' => $values['usePrivateWikiScript'],
       'privateWikiScriptUrl' => $values['privateWikiScriptUrl'],
       'useCustomDomainScript' => $values['useCustomDomainScript'],
       'useCustomDomainScriptSecure' => $values['useCustomDomainScriptSecure'],
       'login' => $values['login'],
       'pageOptions' => $values['pageOptions'],
   ]);
});

Route::get('/user--services/logout', [AuthenticatedSessionController::class, 'destroy']);

Route::middleware(['auth:sanctum', 'verified'])->get('/user--services/dashboard', function () {
    return view('dashboard');
})->name('dashboard');

if (GlobalProperties::$FEATURE_FRONTEND === 'next') {
    // Legacy special routes
    Route::any("/{special}:{path}", [OzoneController::class, 'handle'])
        ->where("special", "system|admin")
        ->where("path", ".+");

    Route::get('/{path?}', function () {
        $values = LegacyTools::generateScreenVars();

        // TODO: move this to a controller

        $title = 'Wikijump';
        $canonical = URL::current();
        $license = null;

        $social_title = null;

        $slug = null;
        $category = null;
        $breadcrumbs = null;
        $page_title = null;
        $revision = null;
        $last_edit_date = null;
        $last_edit_days_since = null;
        $tags = null;

        if ($values['wikiPage']) {
            $page = $values['wikiPage'];
            $timestamp = $page->getDateLastEdited()->getTimestamp();

            $slug = $page->getUnixName();
            $page_title = $page->getTitleOrUnixName();
            $revision = $page->getRevisionNumber();
            $last_edit_date = strftime('%x %r', $timestamp);
            $last_edit_days_since = floor((time() - $timestamp) / (60 * 60 * 24));
            $tags = $values['tags'] ?? null;

            $title = $page_title . ' | ' . $title;

            $social_title = $page_title;

            // this should always be there, but just in case...
            if ($values['category']) {
                $category = $values['category']->getName();

                // we only want to provide license info if the page actually has one
                $lic = $values['category']->getLicense();
                if ($lic->url()) {
                    $license = $lic;
                }
            }

            if ($values['breadcrumbs']) {
                $breadcrumbs = [];
                foreach ($values['breadcrumbs'] as $breadcrumb) {
                    array_push($breadcrumbs, [
                        'title' => $breadcrumb->getTitleOrUnixName(),
                        'slug' => $breadcrumb->getUnixName(),
                    ]);
                }
            }
        }

        return view('next.wiki.page', [
            // TODO: description, image, twitter, etc.
            // TODO: site theming
            // TODO: favicons
            // TODO: header image/text + subtitle management

            'title' => $title,
            'canonical' => $canonical,
            'license' => $license,

            'social_title' => $social_title,
            'social_type' => 'article',
            'social_url' => $canonical,

            'header_img_url' => '/files--static/media/logo-outline.min.svg',

            'navbar_items' => [
                'SCP Series' => [
                    'Series VII' => '/',
                    'Series VI' => '/',
                    'Series V' => '/',
                    'Series IV' => '/',
                    'Series III' => '/',
                    'Series II' => '/',
                    'Series I' => '/',
                ],
                'Tales' => [
                    'Foundation Tales' => '/',
                    'Series Archive' => '/',
                    'Incident Reports' => '/',
                    'CreepyPasta Archive' => '/',
                ],
                'Library' => [],
                'Universe' => [],
                'SCP Global' => [],
                'Background' => "/",
                'About' => "/",
            ],

            'sidebar_content' => $values['sideBar1Content'] ?? null,
            'page_content' => $values['pageContent'] ?? null,

            'page_slug' => $slug,
            'page_category' => $category,
            'page_title' => $page_title,
            'page_breadcrumbs' => $breadcrumbs,
            'page_revision' => $revision,
            'page_last_edit_date' => $last_edit_date,
            'page_last_edit_days_since' => $last_edit_days_since,
            'page_tags' => $tags,

            'HTTP_SCHEMA' => GlobalProperties::$HTTP_SCHEMA,
            'URL_DOMAIN' => GlobalProperties::$URL_DOMAIN,
            'URL_HOST' => GlobalProperties::$URL_HOST,
            'SERVICE_NAME' => GlobalProperties::$SERVICE_NAME,
        ]);
    })->where( "path", ".*" );
} else {
    /**
     * This fallback route will defer to the OzoneController, which will boot an
     * instance of the legacy WikiFlowController and let it handle the response.
     * Significantly, since the request is being run through Laravel and Ozone's
     * involvement is reduced to that of a controller, the full set of Laravel
     * Models, Facades, and helpers are available everywhere in the codebase.
     */
    Route::any( "/{path?}", [OzoneController::class, 'handle'] )
        ->where( "path", ".*" );
}


/** Use blade for everything. Soon™. */
//Route::any( "/{path?}", function() {
//    $values = LegacyTools::generateScreenVars();
//    return view(
//        'layouts.legacy',
//        [
//            'site' => $values['site'] ?? null,
//            'pageNotExists' => $values['pageNotExists'] ?? null,
//            'category' => $values['category'] ?? null,
//            'theme' => $values['theme'] ?? null,
//            'wikiPage' => ($values['wikiPage'] ?? null),
//            'wikiPageName' => ($values['wikiPageName'] ?? null),
//            'pageContent' => ($values['pageContent'] ?? null),
//            'pageParameters' => ($values['pageParameters'] ?? null),
//            'topBarContent' => $values['topBarContent'] ?? null,
//            'sideBar1Content' => $values['sideBar1Content'] ?? null,
//            'breadcrumbs' => $values['breadcrumbs'] ?? null,
//            'tags' => $values['tags'] ?? null,
//            'licenseHtml' => $values['licenseHtml'] ?? null,
//            'HTTP_SCHEMA' => GlobalProperties::$HTTP_SCHEMA,
//            'URL_DOMAIN' => GlobalProperties::$URL_DOMAIN,
//            'URL_HOST' => GlobalProperties::$URL_HOST,
//            'SERVICE_NAME' => GlobalProperties::$SERVICE_NAME,
//            'usePrivateWikiScript' => $values['usePrivateWikiScript'],
//            'privateWikiScriptUrl' => $values['privateWikiScriptUrl'],
//            'useCustomDomainScript' => $values['useCustomDomainScript'],
//            'useCustomDomainScriptSecure' => $values['useCustomDomainScriptSecure'],
//            'login' => $values['login'],
//            'pageOptions' => $values['pageOptions'],
//
//        ]
//    );
//})->where("path", ".*");
