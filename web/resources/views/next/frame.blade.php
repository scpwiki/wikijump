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
        $plain (if true, no navbar, no sidebar, no license)

    sections:
        content
--}}

@extends('next.base')

@section('app')
    @if (empty($plain) || !$plain)
        <div id="app" @class([
            'has-sidebar' => isset($sidebar_content),
            'has-license' => isset($license),
        ])>

            @include('next.components.header')

            @include('next.components.navbar')

            @includeWhen(isset($sidebar_content), 'next.components.sidebar')

            <main id="main" aria-label="{{ __('main-content') }}">
                @yield('content')
            </main>

            @include('next.components.footer')
        </div>
    @else
        <div id="app" class="is-plain">

            @include('next.components.header')

            <main id="main" aria-label="{{ __('main-content') }}">
                @yield('content')
            </main>

            @include('next.components.footer')
        </div>
    @endif
@endsection
