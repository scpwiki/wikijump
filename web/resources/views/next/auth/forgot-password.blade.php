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

    <wj-component-loader load="ForgotPasswordForm" skeleton="spinner:6rem">
    </wj-component-loader>
@endsection
