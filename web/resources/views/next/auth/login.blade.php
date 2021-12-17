{{--
    Login screen (non-API).
    Extends from `next.auth.auth`.
--}}

@extends('next.auth.auth', [
    'title' => __('auth.LOGIN')
])

@section('content')
    <wj-component-loader load="LoginForm" back="{{ previousUrl() }}">
    </wj-component-loader>

    <a id="auth_create_account" href="/user--services/register">
        {{ __("auth.CREATE_ACCOUNT") }}
    </a>
@endsection
