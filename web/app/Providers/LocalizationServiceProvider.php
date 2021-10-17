<?php
declare(strict_types=1);

namespace Wikijump\Providers;

use Illuminate\Support\ServiceProvider;
use Wikijump\Services\Localization\LocalizationService;

class LocalizationServiceProvider extends ServiceProvider
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
        LocalizationService::setup();
    }
}
