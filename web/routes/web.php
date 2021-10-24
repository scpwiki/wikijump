<?php

use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Facades\Route;
use Laravel\Fortify\Http\Controllers\AuthenticatedSessionController;
use Ozone\Framework\Ozone;
use Ozone\Framework\RunData;
use Wikidot\Utils\AjaxModuleWikiFlowController;
use Wikidot\Utils\GlobalProperties;
use Wikijump\Http\Controllers\AuthController;
use Wikijump\Http\Controllers\OzoneController;
use Wikijump\Http\Controllers\PageController;
use Wikijump\Models\User;

use const Wikijump\Helpers\LegacyTools;

/**
 * We are instantiating a RunData object with every route request so we can pull
 * some information related to sessions. Hopefully this won't stay in for long.
 */
if (php_sapi_name() !== 'cli') {
    Ozone::init();
    $runData = new RunData();
    $runData->init();
    Ozone::setRunData($runData);
    $runData->handleSessionStart();
    if ($runData->getUserId() && Auth::guest()) {
        Auth::login(User::findorFail($runData->getUserId()));
    }
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

Route::get('/page--test', function () {
    return view('next.test.page-test', [
        'header_img_url' => '/files--static/media/logo-outline.min.svg',
    ]);
});
/**
 * Socialite route, null until I'm ready to begin work there.
 */
Route::prefix('social--providers')->group(function () {
    Route::get('/callback', function ($provider) {
        return app()->call('Wikijump\Http\Controllers\SocialiteController@callback', [
            'provider' => $provider,
        ]);
    })->name('socialite-callback');
});

/**
 * AJAX Handler, formerly ajax-module-connector.php
 */
Route::post('/ajax--handler', function () {
    $controller = new AjaxModuleWikiFlowController();
    $controller->process();
});

/**
 * Karma displayer.
 */
Route::get('/user--karma/{user}', function (User $user) {
    // prettier-ignore
    $karma = Cache::remember('karma_level__user_'.$user->id, 3600, function() use($user) {
        return $user->karma_level;
    });
    header('Content-type: image/png');
    header('Expires: ' . gmdate('D, d M Y H:i:s', time() + 3600) . ' GMT');
    header('Last-Modified: ' . gmdate('D, d M Y H:i:s', time()) . ' GMT');
    header('Cache-Control: max-age=3600, must-revalidate');
    // prettier-ignore
    readfile(WIKIJUMP_ROOT.'/web/files--common/theme/base/images/karma/karma_'.$karma.'.png');
});

/**
 * Avatar shortcut
 */
Route::get('/user--avatar/{user}', function (User $user) {
    return $user->avatar();
});

// -- USER SERVICES

// TODO: password.request
// TODO: password.reset
// TODO: password.email
// TODO: password.update

// TODO: emails
// TODO: two factor

// Auth Routes
Route::prefix('user--services')
    ->middleware(['auth', 'verified'])
    ->group(function () {
        // TODO: remove this temporary helper route
        Route::get('/logout', [AuthController::class, 'logout']);

        // name is important, it's reserved by Laravel for password confirmation
        Route::view('/confirm-password', 'next.auth.confirm-password')->name(
            'password.confirm',
        );
    });

// Guest routes
Route::prefix('user--services')
    ->middleware('guest')
    ->group(function () {
        Route::view('/login', 'next.auth.login')->name('login');
        Route::view('/register', 'next.auth.register')->name('register');
    });

// -- WIKI

if (GlobalProperties::$FEATURE_FRONTEND === 'next') {
    // Legacy special routes
    Route::any('/{special}:{path}', [OzoneController::class, 'handle'])
        ->where('special', 'system|admin')
        ->where('path', '.+');

    Route::get('/{path?}', [PageController::class, 'show'])->where('path', '.*');
} else {
    /**
     * This fallback route will defer to the OzoneController, which will boot an
     * instance of the legacy WikiFlowController and let it handle the response.
     * Significantly, since the request is being run through Laravel and Ozone's
     * involvement is reduced to that of a controller, the full set of Laravel
     * Models, Facades, and helpers are available everywhere in the codebase.
     */
    Route::any('/{path?}', [OzoneController::class, 'handle'])->where('path', '.*');
}
