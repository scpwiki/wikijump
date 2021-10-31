<?php

declare(strict_types=1);

namespace Wikijump\Providers;

use Illuminate\Support\Facades\View;
use Illuminate\Support\ServiceProvider;
use Wikijump\View\Composers\BaseComposer;
use Wikijump\View\Composers\PageMockedComposer;
use Wikijump\View\Directives\PreloadDirective;

class ViewServiceProvider extends ServiceProvider
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
        View::composer('next.base', BaseComposer::class);
        View::composer('next.test.page-test', PageMockedComposer::class);

        PreloadDirective::register();
    }
}
