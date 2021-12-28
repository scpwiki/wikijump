{{--
    Password recovery form.
    Extends from `next.auth.auth`.
--}}

@extends('next.auth.auth', [
    'title' => __('forgot-password'),
])

@section('content')
    <h1 id="auth_title">
        {{ __('forgot-password') }}
    </h1>

    <wj-component-loader ld-load="ForgotPasswordForm" ld-skeleton="spinner:6rem">
    </wj-component-loader>
@endsection
