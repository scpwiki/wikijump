{{--
    Password recovery form.
    Extends from `next.auth.auth`.
--}}

@extends('next.auth.auth', [
    'title' => __('auth.password_recovery.FORGOT_PASSWORD'),
])

@push('preloads')
    @preload('auth-forgot-password.ts')
@endpush

@push('scripts')
    @vite('auth-forgot-password.ts')
@endpush

@section('content')
    <h1 id="auth_title">
        {{ __('auth.password_recovery.FORGOT_PASSWORD') }}
    </h1>

    <div id="auth_form_container">
    </div>
@endsection
