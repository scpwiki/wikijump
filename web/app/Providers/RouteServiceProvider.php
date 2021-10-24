<?php
declare(strict_types=1);

namespace Wikijump\Providers;

use Illuminate\Cache\RateLimiting\Limit;
use Illuminate\Foundation\Support\Providers\RouteServiceProvider as ServiceProvider;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\RateLimiter;
use Illuminate\Support\Facades\Route;

/**
 * Service Provider for route publishing and rate limits.
 *
 * @package Wikijump\Providers
 */
class RouteServiceProvider extends ServiceProvider
{
    /**
     * The path to the "home" route for your application.
     *
     * This is used by Laravel authentication to redirect users after login.
     *
     * @var string
     */
    public const HOME = '/';

    /**
     * The controller namespace for the application.
     *
     * When present, controller route declarations will automatically be prefixed with this namespace.
     *
     * @var string|null
     */
    // protected $namespace = 'Wikijump\\Http\\Controllers';

    /**
     * Define your route model bindings, pattern filters, etc.
     *
     * @return void
     */
    public function boot()
    {
        $this->configureRateLimiting();

        $this->routes(function () {
            // We may need to use this for pure API stuff,
            // e.g. CROM or other scraping tools.
            // I think `/query` will probably go in here.
            // Since nothing uses it right now, I gave it a
            // path that shouldn't conflict with anything.
            Route::prefix('api--unused')
                ->middleware('api')
                ->namespace($this->namespace)
                ->group(base_path('routes/api.php'));

            Route::prefix('api--v0')
                ->middleware('web')
                ->namespace($this->namespace)
                ->group(base_path('routes/api-v0.php'));

            Route::middleware('web')
                ->namespace($this->namespace)
                ->group(base_path('routes/web.php'));
        });

        Route::pattern('user', '[1-9][0-9]*');
    }

    /**
     * Configure the rate limiters for the application.
     *
     * @return void
     */
    protected function configureRateLimiting()
    {
        RateLimiter::for('api', function (Request $request) {
            return Limit::perMinute(60)->by(
                optional($request->user())->id ?: $request->ip(),
            );
        });
    }

    /**
     * Used to generate routes that aren't already cached.
     *
     * @return void
     */
    public function map(): void
    {
        //        $this->mapApiRoutes();
        //        $this->mapWebRoutes();
        $this->mapServiceRoutes();
    }

    /**
     * Create needed routes for services, as config is processed before booting
     * the app.
     *
     * @return void
     */
    protected function mapServiceRoutes(): void
    {
        /**
         * Socialite routes.
         */
        config()->set(
            'services.facebook.redirect',
            route('socialite-callback', ['provider' => 'facebook']),
        );
        config()->set(
            'services.twitter.redirect',
            route('socialite-callback', ['provider' => 'twitter']),
        );
        config()->set(
            'services.google.redirect',
            route('socialite-callback', ['provider' => 'google']),
        );
    }
}
