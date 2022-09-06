<?php

use Illuminate\Foundation\Auth\EmailVerificationRequest;
use Illuminate\Support\Facades\Route;
use Wikijump\Http\Controllers\AccountController;
use Wikijump\Http\Controllers\AuthController;
use Wikijump\Models\User;

// TODO: two factor

// Guest routes
Route::middleware('guest')->group(function () {
    Route::view('/login', 'next.auth.login')->name('login');

    Route::view('/register', 'next.auth.register')->name('register');

    // Passsword reset routes

    Route::view('/forgot-password', 'next.auth.forgot-password')->name(
        'password.request',
    );

    Route::view('/reset-password/{token}', 'next.auth.reset-password')->name(
        'password.reset',
    );

    Route::post('/reset-password/{token}', [
        AccountController::class,
        'handlePasswordRecoveryUpdate',
    ])->name('password.update');
});

// Auth (unverified) routes
Route::middleware('auth')->group(function () {
    // Email verification routes

    Route::get('/verify-email', function (User $user) {
        // don't allow showing the verification notice if the user is already verified
        if ($user->hasVerifiedEmail()) {
            return redirect('/');
        }

        return view('next.auth.verify-email');
    })->name('verification.notice');

    // this route isn't signed, but the `POST` version of it is
    Route::view('/verify-email/{id}/{hash}', 'next.auth.verify-email-link');

    Route::post('/verify-email/{id}/{hash}', function (
        EmailVerificationRequest $request
    ) {
        $request->fulfill();
        return response('', 200);
    })
        ->middleware('signed')
        ->name('verification.verify');

    // TODO: remove this temporary helper route
    Route::get('/logout', [AuthController::class, 'logout'])->name('logout');

    // Verified routes
    Route::middleware('verified')->group(function () {
        // name is important, it's reserved by Laravel for password confirmation
        Route::view('/confirm-password', 'next.auth.confirm-password')->name(
            'password.confirm',
        );
    });
});
