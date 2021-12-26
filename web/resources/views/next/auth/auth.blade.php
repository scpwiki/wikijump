{{--
    Base view for authentication handling.
    Extends from `next.base`.

    sections:
        content
--}}

@extends('next.base')

@push('preloads')
    @preload('auth.scss')
@endpush

@push('styles')
    @vite('auth.scss')
@endpush

@section('app')
    <div id="app_auth">
        <div id="auth_panel" class="light">
            <a href="/" title="{{ __('goto-home') }}">
                <img src="/files--static/media/logo.min.svg">
            </a>
            <hr>

            @yield('content')

            {{-- gets placed _outside_ of the panel via styling --}}
            <div id="auth_links">
                <a href="{{ route("terms") }}">{{ __('terms') }}</a>
                <a href="{{ route("privacy") }}">{{ __('privacy') }}</a>
                <a href="{{ route("docs") }}">{{ __('docs') }}</a>
                <a href="{{ route("security") }}">{{ __('security') }}</a>
            </div>
        </div>
    </div>
@endsection
