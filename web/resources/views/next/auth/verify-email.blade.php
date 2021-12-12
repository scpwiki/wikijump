{{--
    View shown to users who haven't verified their email address.
    Extends from 'next.auth.auth'
--}}

@extends('next.auth.auth', [
    'title' => __('account_panel.verify_email.VERIFY_EMAIL')
])

@push('preloads')
    @preload('resources/scripts/auth-verify-email.ts')
@endpush

@push('scripts')
    @vite('auth-verify-email.ts')
@endpush

@section('content')
    <p id="auth_verify_email">
        {{ __('account_panel.verify_email.INTRO') }}
    </p>
    <br />
    <div id="auth_form_container"></div>
@endsection
