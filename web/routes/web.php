<?php

use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Facades\Route;
use Ozone\Framework\Ozone;
use Ozone\Framework\RunData;
use Wikijump\Http\Controllers\OzoneController;
use Wikijump\Models\User;

/**
 * We are instantiating a RunData object with every route request so we can pull
 * some information related to sessions. Hopefully this won't stay in for long.
 */
Ozone ::init();
$runData = new RunData();
$runData->init();
Ozone :: setRunData($runData);
$runData->handleSessionStart();
//   return \Illuminate\Support\Facades\Auth::user();
if($runData->getUserId() && Auth::guest()) {
    Auth::login(
        User::findorFail($runData->getUserId())
    );
};




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
 * This fallback route will defer to the OzoneController, which will boot an
 * instance of the legacy WikiFlowController and let it handle the response.
 * Significantly, since the request is being run through Laravel and Ozone's
 * involvement is reduced to that of a controller, the full set of Laravel
 * Models, Facades, and helpers are available everywhere in the codebase.
 */
Route::any( "/{path?}", [OzoneController::class, 'handle'] )
    ->where( "path", ".*" );

