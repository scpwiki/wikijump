{{--
    Confirm password view.
    Extends from `next.auth.auth`.
--}}

@extends('next.auth.auth', [
    'title' => __('confirm-password')
])

@section('content')
    <h1 id="auth_title">
        {{ __('confirm-password') }}
    </h1>

    <wj-component-loader
        ld-load="ConfirmPasswordForm"
        ld-skeleton="spinner:6rem"
        back="{{ previousUrl() }}"
    >
    </wj-component-loader>

    <a id="auth_forgot_password" href="{{ route("password.request") }}">
        {{ __("forgot-password.question") }}
    </a>
@endsection
