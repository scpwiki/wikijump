<?php
declare(strict_types=1);

namespace Wikijump\Providers;

use Illuminate\Contracts\Auth\StatefulGuard;
use Illuminate\Foundation\Support\Providers\AuthServiceProvider as ServiceProvider;
use Illuminate\Support\Facades\Auth;

class AuthServiceProvider extends ServiceProvider
{
    /**
     * The policy mappings for the application.
     *
     * @var array
     */
    protected $policies = [
        // 'Wikijump\Models\Model' => 'Wikijump\Policies\ModelPolicy',
    ];

    /**
     * Register any application services.
     *
     * @return void
     */
    public function register()
    {
        $this->app->bind(StatefulGuard::class, function () {
            return Auth::guard('web');
        });
    }

    /**
     * Register any authentication / authorization services.
     *
     * @return void
     */
    public function boot()
    {
        $this->registerPolicies();
    }
}
