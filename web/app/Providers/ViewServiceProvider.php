<?php

declare(strict_types=1);

namespace Wikijump\Providers;

use Illuminate\Support\Facades\View;
use Illuminate\Support\ServiceProvider;
use Wikijump\View\Composers\PageMockedComposer;

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
        View::composer('next.test.page-test', PageMockedComposer::class);
    }
}
