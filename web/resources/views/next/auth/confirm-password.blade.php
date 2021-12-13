{{--
    Confirm password view.
    Extends from `next.auth.auth`.
--}}

@extends('next.auth.auth', [
    'title' => __('auth.CONFIRM_PASSWORD')
])

@push('preloads')
    @preload('auth-confirm.ts')
@endpush

@push('scripts')
    @vite('auth-confirm.ts')
@endpush

@section('content')
    <h1 id="auth_title">
        {{ __('auth.CONFIRM_PASSWORD') }}
    </h1>

    <div id="auth_form_container">
    </div>

    <a id="auth_forgot_password" href="/user--services/forgot-password">
        {{ __("auth.FORGOT_PASSWORD") }}
    </a>
@endsection
