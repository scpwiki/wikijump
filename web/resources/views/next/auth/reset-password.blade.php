{{--
    Password reset form.
    Extends from `next.auth.auth`.
--}}

@extends('next.auth.auth', [
    'title' => __('auth.password_recovery.RESET_PASSWORD'),
])

@section('content')
    <h1 id="auth_title">
        {{ __('auth.password_recovery.RESET_PASSWORD') }}
    </h1>

    <wj-component-loader load="ResetPasswordForm" goto="{{ route('login') }}">
    </wj-component-loader>
@endsection
