{{--
    Password recovery form.
    Extends from `next.auth.auth`.
--}}

@extends('next.auth.auth', [
    'title' => __('auth.password_recovery.FORGOT_PASSWORD'),
])

@section('content')
    <h1 id="auth_title">
        {{ __('auth.password_recovery.FORGOT_PASSWORD') }}
    </h1>

    <wj-component-loader load="ForgotPasswordForm">
    </wj-component-loader>
@endsection
