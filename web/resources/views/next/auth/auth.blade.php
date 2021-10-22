{{--
    Base view for authentication handling.
    Extends from `next.base`.

    sections:
        content
--}}

@extends('next.base')

@section('app')
    <div id="app_auth">
        <div id="auth_panel" class="light">
            <a href="/" title="{{ __('frame.GOTO_HOME_PAGE') }}">
                <img src="/files--static/media/logo.min.svg">
            </a>
            <hr>

            @yield('content')

            {{-- gets placed _outside_ of the panel via styling --}}
            <div id="auth_links">
                {{-- TODO: link to actual pages --}}
                <a href="/terms">{{ __('frame.footer.TERMS') }}</a>
                <a href="/privacy">{{ __('frame.footer.PRIVACY') }}</a>
                <a href="/docs">{{ __('frame.footer.DOCS') }}</a>
                <a href="/security">{{ __('frame.footer.SECURITY') }}</a>
            </div>
        </div>
    </div>
@endsection
