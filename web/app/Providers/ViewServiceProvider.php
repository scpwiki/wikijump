<?php

declare(strict_types=1);

namespace Wikijump\Providers;

use Illuminate\Auth\Notifications\ResetPassword;
use Illuminate\Auth\Notifications\VerifyEmail;
use Illuminate\Support\Facades\View;
use Illuminate\Support\ServiceProvider;
use Wikijump\Mail\PasswordResetMessage;
use Wikijump\Mail\VerifyEmailMessage;
use Wikijump\View\Composers\BaseComposer;
use Wikijump\View\Composers\EmailBaseComposer;
use Wikijump\View\Composers\PageMockedComposer;
use Wikijump\View\Directives\InlineDirective;
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
        // Register Composers

        View::composer('next.base', BaseComposer::class);
        View::composer('emails.base-mjml', EmailBaseComposer::class);
        View::composer('emails.base-text', EmailBaseComposer::class);
        View::composer('next.test.page-test', PageMockedComposer::class);

        // Register Directives

        PreloadDirective::register();
        InlineDirective::register();

        // Override default mailables

        ResetPassword::toMailUsing(function ($notifiable, $token) {
            $email = $notifiable->getEmailForPasswordReset();
            $url = url(
                route('password.reset', ['email' => $email, 'token' => $token], false),
            );
            $expires = config(
                'auth.passwords.' . config('auth.defaults.passwords') . '.expire',
            );

            return new PasswordResetMessage($url, $expires);
        });

        VerifyEmail::toMailUsing(function ($notifiable, $verificationUrl) {
            return new VerifyEmailMessage($verificationUrl);
        });
    }
}
