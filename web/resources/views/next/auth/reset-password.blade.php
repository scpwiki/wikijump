{{--
    Password reset form.
    Extends from `next.auth.auth`.
--}}

@extends('next.auth.auth', [
    'title' => __('reset-password'),
])

@section('content')
    <h1 id="auth_title">
        {{ __('reset-password') }}
    </h1>

    <wj-component-loader
        ld-load="ResetPasswordForm"
        ld-skeleton="spinner:6rem"
        goto="{{ route('login') }}"
    >
    </wj-component-loader>
@endsection
