<?php
declare(strict_types=1);

namespace Wikijump\Providers;

use Exception;
use Illuminate\Support\Facades\Log;
use Illuminate\Support\ServiceProvider;

class AppServiceProvider extends ServiceProvider
{
    /**
     * Register any application services.
     *
     * @return void
     */
    public function register()
    {
        // we don't build the client assets in the php-fpm container,
        // so we'll need to retrieve the manifest for them from the
        // nginx container

        $vite_manifest_path = config('vite.build_path') . '/manifest.json';

        if (!file_exists(public_path($vite_manifest_path))) {
            // get manifest from nginx container
            // we have to catch a potential exception here, because
            // nginx may or may not be running.
            // also, if the manifest isn't there laravel-vite will just
            // throw its own error, so we'll just let it handle that
            try {
                $file = file_get_contents("http://nginx:80/$vite_manifest_path");

                if ($file) {
                    file_put_contents(public_path($vite_manifest_path), $file);
                    Log::debug('Wrote Vite manifest file (retrieved from nginx)');
                }
            } catch (Exception $err) {
            }
        }
    }

    /**
     * Bootstrap any application services.
     *
     * @return void
     */
    public function boot()
    {
        /**
         * We set this because the Ozone codebase has a *lot* of notice-level
         * events that they already set to ignore. Our time is better spent
         * replacing that code instead of bringing it up to scratch.
         */
        error_reporting(E_ALL & ~E_NOTICE);
    }
}
