<?php
declare(strict_types=1);

namespace Wikijump\Providers;

use Illuminate\Support\Facades\Log;
use Illuminate\Support\ServiceProvider;
use Laravel\Telescope\TelescopeServiceProvider;
use Wikijump\Services\NGINX\NGINX;

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
            $contents = NGINX::fetch($vite_manifest_path);
            if ($contents) {
                file_put_contents(public_path($vite_manifest_path), $contents);
                Log::debug('Wrote Vite manifest file (retrieved from nginx)');
            }
        }

        // register Telescope
        if ($this->app->environment('local')) {
            $this->app->register(TelescopeServiceProvider::class);
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
