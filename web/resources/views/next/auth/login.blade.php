{{--
    Login screen (non-API).
    Extends from `next.auth.auth`.
--}}

@extends('next.auth.auth', [
    'title' => __('login')
])

@section('content')
    <wj-component-loader
        load="LoginForm"
        skeleton="inline:5:1.5rem"
        back="{{ previousUrl() }}"
    >
    </wj-component-loader>

    <a id="auth_create_account" href="{{ route("register") }}">
        {{ __("create-account") }}
    </a>
@endsection
