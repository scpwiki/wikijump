<?php
declare(strict_types=1);

namespace Wikijump\Providers;

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
