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
        <header id="header" aria-label="{{ __('frame.aria_header') }}">
            @if (isset($header_img_url) || isset($header_title))
                <a id="header_logo" href="/" title="{{ __('frame.goto_home_page') }}">
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
        <nav id="navbar" aria-label="{{ __('frame.aria_navigation') }}">
            @includeWhen(isset($navbar_items), 'next.components.navbar-elements', [
                'items' => $navbar_items,
            ])
        </nav>


        {{-- Sidebar --}}
        @isset($sidebar_content)
            <aside id="sidebar" aria-label="{{ __('frame.aria_sidebar') }}">
                <div id="sidebar_sticky" role="presentation">
                    {!! $sidebar_content !!}
                </div>
            </aside>
        @endisset

        {{-- Main Content --}}
        <main id="main" aria-label="{{ __('frame.aria_main') }}">
            @yield('content')
        </main>

        {{-- Footer --}}
        <footer id="footer" aria-label="{{ __('frame.aria_footer') }}">
            <div id="footer_main">
                <div id="footer_services">
                    @if ($SERVICE_NAME != "")
                        <a href="{{$HTTP_SCHEMA}}://{{$URL_HOST}}">
                            {{ __('frame.part_of', ['name' => $SERVICE_NAME]) }}
                        </a>
                        <span class="footer_services_sep">&#8212;</span>
                    @endif
                    <a href="https://github.com/scpwiki/wikijump">
                        {{ __('frame.powered_by', ['name' => 'Wikijump']) }}
                    </a>
                    <span class="footer_services_sep">&#8212;</span>
                    {{-- TODO: link to actual pages --}}
                    <a href="/terms">{{ __('frame.terms') }}</a>
                    <a href="/privacy">{{ __('frame.privacy') }}</a>
                    <a href="/docs">{{ __('frame.docs') }}</a>
                </div>

                <div id="footer_actions">
                    <a href="https://scuttle.atlassian.net/servicedesk/customer/portal/2">
                        {{ __('frame.report_bug') }}
                    </a>
                    {{-- TODO: Flag as objectionable functionality  --}}
                    <a href="/flag">
                        {{ __('frame.report_flag') }}
                    </a>
                </div>
            </div>

            @isset($license)
                <div id="footer_license" aria-label="{{ __('frame.aria_license') }}">
                    <a href="{{ $license->url() }}">
                        @if ($license->unless())
                            {{ __('frame.license_unless', ['license' => $license->name()]) }}
                        @else
                            {{  __('frame.license', ['license' => $license->name()]) }}
                        @endif
                    </a>
                </div>
            @endisset
        </footer>
    </div>
@endsection
