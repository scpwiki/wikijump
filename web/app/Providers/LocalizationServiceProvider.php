<?php
declare(strict_types=1);

namespace Wikijump\Providers;


use Illuminate\Support\ServiceProvider;

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
        // Set up gettext
        bindtextdomain('wikijump', WIKIJUMP_ROOT . '/web/resources/lang');
        bind_textdomain_codeset('wikijump', 'UTF-8');
        textdomain('wikijump');
    }
}
