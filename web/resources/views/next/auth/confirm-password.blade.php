{{--
    Confirm password view.
    Extends from `next.auth.auth`.
--}}

@extends('next.auth.auth', [
    'title' => __('auth.CONFIRM_PASSWORD')
])

@section('content')
    <h1 id="auth_title">
        {{ __('auth.CONFIRM_PASSWORD') }}
    </h1>

    <wj-component-loader load="ConfirmPasswordForm" back="{{ previousUrl() }}">
    </wj-component-loader>

    <a id="auth_forgot_password" href="{{ route("password.request") }}">
        {{ __("auth.FORGOT_PASSWORD") }}
    </a>
@endsection
