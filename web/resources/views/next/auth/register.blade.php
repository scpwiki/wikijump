{{--
    Register screen (non-API).
    Extends from `next.auth.auth`.
--}}

@extends('next.auth.auth', [
    'title' => __('auth.REGISTER')
])

@section('content')
    <wj-component-loader load="RegisterForm" goto="{{ route("verification.notice") }}">
    </wj-component-loader>

    <a id="auth_login" href="/user--services/login">
        {{ __("auth.LOGIN") }}
    </a>
@endsection
