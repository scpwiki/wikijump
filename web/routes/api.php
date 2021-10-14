<?php

use Illuminate\Http\Request;
use Illuminate\Support\Facades\App;
use Illuminate\Support\Facades\Http;
use Illuminate\Support\Facades\Route;

/*
|--------------------------------------------------------------------------
| API Routes
|--------------------------------------------------------------------------
|
| Here is where you can register API routes for your application. These
| routes are loaded by the RouteServiceProvider within a group which
| is assigned the "api" middleware group. Enjoy building your API!
|
*/

Route::middleware('auth:sanctum')->get('/user', function (Request $request) {
    return $request->user();
});

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

            return response($res->body(), $res->status(), $res->headers());
        } catch (Exception $err) {
            // server probably isn't up, as otherwise we would've gotten a 404
            return response(null, 404);
        }
    })->where('path', '.*');
}
