{{--
    All arguments are optional.

    data:
        $social_url
        $social_title
        $social_description
        $social_image
        $social_type
        $social_twitter_card
        $social_twitter_site
        $social_twitter_creator
        $social_rating

        $title
        $robots
        $canonical
        $license (see `config/licenses.php` and `App/Services/License`)

        $theme_color

        $favicon_svg
        $favicon_png
        $favicon_apple
        $favicon_mask

    sections:
        app

    stacks:
        preloads
        styles
        head
        scripts
--}}

<!DOCTYPE html>
<html lang="{{ str_replace('_', '-', app()->getLocale()) }}">

<head>
    {{-- Functionality --}}
    <meta charset="utf-8">
    <base href="/">
    <meta name="viewport" content="width=device-width,initial-scale=1">

    {{-- Browser Metadata --}}
    @isset($title)
        <title>{{ $title }}</title>
    @endisset
    @isset($robots)
        <meta name="robots" content="{{ $robots }}">
    @endisset
    @isset($canonical)
        <link rel="canonical" href="{{ $canonical }}">
    @endisset
    @isset($license)
        <link rel="license" href="{{ $license->url() }}">
    @endisset

    {{-- Security --}}
    <meta name="referrer" content="no-referrer">
    <meta name="csrf-token" content="{{ csrf_token() }}" />
    {{-- TODO:
    <meta
        http-equiv="Content-Security-Policy"
        content=""
    > --}}

    {{-- Preloads, Preconnects --}}
    {{-- TODO: figure out how to properly preload these:
        <link rel="preload" href="{{ vite_asset('resources/css/base.scss') }}" as="style">
        <link rel="preload" href="{{ vite_asset('resources/scripts/index.ts') }}"
              as="script">
        <link rel="preload"
            href="{{ vite_asset('resources/fonts/variable/PublicSans-VariableFont.woff2') }}"
            as="font" type="font/woff2" crossorigin>
        <link rel="preload"
            href="{{ vite_asset('resources/fonts/variable/Exo2-VariableFont.woff2') }}"
            as="font" type="font/woff2" crossorigin>
    - --}}
    {{-- TODO: preload the user's locale file --}}
    @stack('preloads')

    {{-- OpenGraph --}}
    {{-- TODO: verify that getLocale returns a well formed string for this --}}
    <meta property="og:locale" content="{{ app()->getLocale() }}">
    @isset($social_url)
        <meta property="og:url" content="{{ $social_url }}">
    @endisset
    @isset($social_title)
        <meta property="og:title" content="{{ $social_title }}">
    @endisset
    @isset($social_description)
        <meta name="description" content="{{ $social_description }}">
        <meta property="og:description" content="{{ $social_description }}">
    @endisset
    @isset($social_image)
        <meta property="og:image" content="{{ $social_image }}">
    @endisset
    @isset($social_type)
        <meta property="og:type" content="{{ $social_type }}">
    @endisset

    {{-- Twitter --}}
    @isset($social_twitter_card)
        <meta name="twitter:card" content="{{ $social_twitter_card }}">
    @endisset
    @isset($social_twitter_site)
        <meta name="twitter:site" content="{{ $social_twitter_site }}">
    @endisset
    @isset($social_twitter_creator)
        <meta name="twitter:creator" content="{{ $social_twitter_creator }}">
    @endisset

    {{-- Social Other --}}
    @isset($rating)
        <meta name="rating" content="{{ $rating }}">
    @endisset

    {{-- Browser Theming --}}
    <meta name="color-scheme" content="light dark">
    @isset($theme_color)
        <meta name="theme-color" content="{{ $theme_color }}">
        <meta name="msapplication-TileColor" content="{{ $theme_color }}">
    @endisset

    {{-- Icons --}}
    @isset($favicon_svg)
        <link rel="icon" type="image/svg+xml" href="{{ $favicon_svg }}">
    @endisset
    @isset($favicon_png)
        <link rel="icon" type="image/png" href="{{ $favicon_png }}">
    @endisset
    @isset($favicon_apple)
        <link rel="apple-touch-icon" href="{{ $favicon_apple }}">
    @endisset
    @isset($favicon_mask)
        @isset($theme_color)
            <link rel="mask-icon" href="{{ $favicon_mask }}"
                  color="{{ $theme_color }}">
        @endisset
        @empty($theme_color)
            <link rel="mask-icon" href="{{ $favicon_mask }}" color="#FFF">
        @endempty
    @endisset

    {{-- Styles --}}
    @stack('styles')

    {{-- Scripts --}}
    {{-- TODO: see if it's possible to make scripts load async --}}
    @vite('index.ts')
    @stack('scripts')

    {{-- Vite --}}
    @client

    {{-- Misc --}}
    <link rel="author" href="humans.txt"> {{-- TODO --}}

    @stack('head')
</head>

<body class="light codetheme-dark">
    @yield('app')

    <div id="toasts"></div>
    <div id="modals"></div>
</body>

</html>
