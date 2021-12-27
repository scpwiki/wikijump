{{--
    View shown to users who haven't verified their email address.
    Extends from 'next.auth.auth'
--}}

@extends('next.auth.auth', [
    'title' => __('wiki-auth-verify-email')
])


@section('content')
    <p id="auth_verify_email">
        {{ __('wiki-auth-verify-email.intro') }}
    </p>
    <br />

    <wj-component-loader load="ResendVerificationEmail" skeleton="spinner:6rem">
    </wj-component-loader>
@endsection
