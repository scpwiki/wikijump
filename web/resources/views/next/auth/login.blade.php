{{--
    Login screen (non-API).
    Extends from `next.auth.auth`.
--}}

@extends('next.auth.auth', [
    'title' => __('account_panel.LOGIN')
])

@push('preloads')
    @preload('auth-login.ts')
@endpush

@push('scripts')
    @vite('auth-login.ts')
@endpush

@section('content')
    <div id="auth_form_container">
    </div>

    <a id="auth_create_account" href="/user--services/register">
        {{ __("account_panel.CREATE_ACCOUNT") }}
    </a>
@endsection
