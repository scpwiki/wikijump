{{--
    Frame that general wiki views inherit from.
    Extends from `next.base`.

    The `$navbar_items` variable needs to have the following structure:
        $navbar_items = [
            'dropdown-name' => [
                'link-name' => 'link-url',
                ...
            ],
            // or
            'link-name' => 'link-url',
            ...
        ];

    data:
        $header_img_url
        $header_title
        $header_subtitle
        $navbar_items
        $sidebar_content (UNESCAPED)

    sections:
        content
--}}

@extends('next.base')

@section('app')
    <div id="app" @class([
        'has-sidebar' => isset($sidebar_content),
        'has-license' => isset($license),
    ])>

        {{-- Header --}}
        {{-- TODO: User account control widget --}}
        <header id="header" aria-label="{{ __('frame.aria.HEADER') }}">
            @if (isset($header_img_url) || isset($header_title))
                <a id="header_logo" href="/" title="{{ __('frame.GOTO_HOME_PAGE') }}">
                    @isset($header_img_url)
                        <img id="header_logo_img"
                             src="{{ $header_img_url }}"
                             aria-hidden="true"
                        >
                    @endisset
                    @isset($header_title)
                        <h1 id="header_logo_title">{{ $header_title }}</h1>
                    @endisset
                    @isset($header_subtitle)
                        <small id="header_logo_subtitle">{{ $header_subtitle }}</small>
                    @endisset
                </a>
            @endif
        </header>

        {{-- Navbar --}}
        {{-- TODO: Page search widget --}}
        {{-- TODO: Locale selector--}}
        {{-- TODO: Dark/light mode selector --}}
        <nav id="navbar" aria-label="{{ __('frame.aria.NAVIGATION') }}">
            @includeWhen(isset($sidebar_content), 'next.components.sidebar-button')

            @includeWhen(isset($navbar_items), 'next.components.navbar-elements', [
                'items' => $navbar_items,
            ])
        </nav>

        {{-- Sidebar --}}
        @includeWhen(isset($sidebar_content), 'next.components.sidebar')

        {{-- Main Content --}}
        <main id="main" aria-label="{{ __('frame.aria.MAIN') }}">
            @yield('content')
        </main>

        {{-- Footer --}}
        <footer id="footer" aria-label="{{ __('frame.aria.FOOTER') }}">
            <div id="footer_main">
                <div id="footer_services">
                    @if ($SERVICE_NAME != "")
                        <a class="footer-services-partof"
                           href="{{$HTTP_SCHEMA}}://{{$URL_HOST}}"
                        >
                            {{ __('frame.footer.PART_OF', ['name' => $SERVICE_NAME]) }}
                        </a>
                        <span class="footer-services-sep">&#8212;</span>
                    @endif
                    <a class="footer-services-poweredby"
                       href="https://github.com/scpwiki/wikijump"
                    >
                        {{ __('frame.footer.POWERED_BY', ['name' => 'Wikijump']) }}
                    </a>
                    <span class="footer-services-sep">&#8212;</span>
                    {{-- TODO: link to actual pages --}}
                    <a href="/terms">{{ __('frame.footer.TERMS') }}</a>
                    <a href="/privacy">{{ __('frame.footer.PRIVACY') }}</a>
                    <a href="/docs">{{ __('frame.footer.DOCS') }}</a>
                </div>

                <div id="footer_actions">
                    <a href="https://scuttle.atlassian.net/servicedesk/customer/portal/2">
                        {{ __('frame.footer.REPORT_BUG') }}
                    </a>
                    {{-- TODO: Flag as objectionable functionality  --}}
                    <a href="/flag">
                        {{ __('frame.footer.REPORT_FLAG') }}
                    </a>
                </div>
            </div>

            @isset($license)
                <div id="footer_license" aria-label="{{ __('frame.aria.LICENSE') }}">
                    <a href="{{ $license->url() }}">
                        @if ($license->unless())
                            {{ __('frame.LICENSE_UNLESS', ['license' => $license->name()]) }}
                        @else
                            {{  __('frame.LICENSE', ['license' => $license->name()]) }}
                        @endif
                    </a>
                </div>
            @endisset
        </footer>
    </div>
@endsection
