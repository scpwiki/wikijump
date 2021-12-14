{{--
    Password reset form.
    Extends from `next.auth.auth`.
--}}

@extends('next.auth.auth', [
    'title' => __('auth.password_recovery.RESET_PASSWORD'),
])

@push('preloads')
    @preload('auth-reset-password.ts')
@endpush

@push('scripts')
    @vite('auth-reset-password.ts')
@endpush

@section('content')
    <h1 id="auth_title">
        {{ __('auth.password_recovery.RESET_PASSWORD') }}
    </h1>

    <div id="auth_form_container">
    </div>
@endsection
