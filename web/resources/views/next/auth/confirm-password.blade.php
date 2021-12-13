{{--
    Confirm password view.
    Extends from `next.auth.auth`.
--}}

@extends('next.auth.auth', [
    'title' => __('account_panel.CONFIRM_PASSWORD')
])

@push('preloads')
    @preload('auth-confirm.ts')
@endpush

@push('scripts')
    @vite('auth-confirm.ts')
@endpush

@section('content')
    <h2 id="auth_confirm_password">
    {{ __('account_panel.CONFIRM_PASSWORD') }}
    </h2>

    <div id="auth_form_container">
    </div>
@endsection
