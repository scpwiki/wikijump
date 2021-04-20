<?php

namespace Wikijump\Providers;

use Illuminate\Support\Facades\Route;
use Illuminate\Support\ServiceProvider;
use Wikidot\Utils\WikiFlowController;

class OzoneServiceProvider extends ServiceProvider
{
    /**
     * Bootstrap any application services.
     *
     * @return void
     */
    public function boot()
    {
        require_once base_path('php/setup.php');
        define('LARAVEL_ENABLED', true);
        $controller = new WikiFlowController();
        $controller->process();

    }
}
