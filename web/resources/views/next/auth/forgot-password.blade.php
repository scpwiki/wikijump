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
    <h2 id="auth_title">
        {{ __('auth.password_recovery.FORGOT_PASSWORD') }}
    </h2>

    <div id="auth_form_container">
    </div>
@endsection
