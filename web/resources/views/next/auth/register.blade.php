{{--
    Register screen (non-API).
    Extends from `next.auth.auth`.
--}}

@extends('next.auth.auth', [
    'title' => __('register')
])

@section('content')
    <wj-component-loader
        load="RegisterForm"
        skeleton="spinner:12rem"
        goto="{{ route("verification.notice") }}"
    >
    </wj-component-loader>

    <a id="auth_login" href="{{ route("login") }}">
        {{ __("login") }}
    </a>
@endsection
