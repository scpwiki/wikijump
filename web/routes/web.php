<?php

use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Facades\Route;
use Ozone\Framework\Ozone;
use Ozone\Framework\RunData;
use Wikidot\Utils\AjaxModuleWikiFlowController;
use Wikijump\Http\Controllers\OzoneController;
use Wikijump\Models\User;

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
 * This fallback route will defer to the OzoneController, which will boot an
 * instance of the legacy WikiFlowController and let it handle the response.
 * Significantly, since the request is being run through Laravel and Ozone's
 * involvement is reduced to that of a controller, the full set of Laravel
 * Models, Facades, and helpers are available everywhere in the codebase.
 */
Route::any( "/{path?}", [OzoneController::class, 'handle'] )
    ->where( "path", ".*" );
