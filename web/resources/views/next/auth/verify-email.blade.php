{{--
    View shown to users who haven't verified their email address.
    Extends from 'next.auth.auth'
--}}

@extends('next.auth.auth', [
    'title' => __('auth.verify_email.VERIFY_EMAIL')
])


@section('content')
    <p id="auth_verify_email">
        {{ __('auth.verify_email.INTRO') }}
    </p>
    <br />

    <wj-component-loader load="ResendVerificationEmail">
    </wj-component-loader>
@endsection
