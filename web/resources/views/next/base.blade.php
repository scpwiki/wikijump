{{--
    All arguments are optional.

    data:
        $title
        $robots
        $canonical
        $license (see `config/licenses.php` and `App/Services/License`)

        $social_url
        $social_title
        $social_description
        $social_image
        $social_type
        $social_twitter_card
        $social_twitter_site
        $social_twitter_creator
        $social_rating

        $theme_color

        $favicon_svg   (.svg)
        $favicon_png   (.png)
        $favicon_apple (.png)
        $favicon_mask  (.svg)

        $HTTP_SCHEMA
        $URL_DOMAIN
        $URL_HOST
        $SERVICE_NAME

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
    {{-- TODO: make this use site name, not $SERVICE_NAME --}}
    <title>{{ isset($title) ? "$title | $SERVICE_NAME" : $SERVICE_NAME }}</title>
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
    @preload('/files--static/fonts/variable/PublicSans-VariableFont.woff2')
    @preload('/files--static/fonts/variable/RedHatDisplayVF.woff2')
    @preload('resources/scripts/index.ts')
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

    {{-- Favicons --}}

    {{--
        SVG favicons are prioritized over PNG favicons.
        Both can be given, the SVG will be used if it exists, with
        the PNG set as a fallback.

        If neither SVG or PNG icons are given, the template will
        use the fallback icons, even if $favicon_apple or $favicon_mask
        are given. These two icons would be very strange defined by themselves.
    --}}

    @if (!isset($favicon_svg) && !isset($favicon_png))
        <link rel="icon" type="image/svg+xml" href="/favicon.svg">
        <link rel="alternate icon" type="image/png" href="/favicon.png">
        <link rel="apple-touch-icon" href="/favicon-apple-touch.png">
        <link rel="mask-icon"
              href="/favicon-mask.svg"
              color="{{ isset($theme_color) ? $theme_color : '#FFF' }}"
        >
    @else
        @isset($favicon_svg)
            <link rel="icon" type="image/svg+xml" href="{{ $favicon_svg }}">
        @endisset

        @if (isset($favicon_png))
            <link rel="{{ isset($favicon_svg) ? "alternate icon" : "icon" }}"
                  type="image/png"
                  href="{{ $favicon_png }}"
            >
        @elseif (isset($favicon_apple))
            {{--
                I don't know why you would do this, but I've added a fallback anyways.
                If you set the apple PNG icon, but not the normal PNG icon, the apple
                icon will be used as the png icon as a falllback.
            --}}
            <link rel="{{ isset($favicon_svg) ? "alternate icon" : "icon" }}"
                  type="image/png"
                  href="{{ $favicon_apple }}"
            >
        @endif

        @if (isset($favicon_apple))
            <link rel="apple-touch-icon" href="{{ $favicon_apple }}">
        @elseif (isset($favicon_png))
            <link rel="apple-touch-icon" href="{{ $favicon_png }}">
        @endif

        @if (isset($favicon_mask))
            <link rel="mask-icon"
                  href="{{ $favicon_mask }}"
                  color="{{ isset($theme_color) ? $theme_color : '#FFF' }}"
            >
        @elseif (isset($favicon_svg))
            <link rel="mask-icon"
                  href="{{ $favicon_svg }}"
                  color="{{ isset($theme_color) ? $theme_color : '#FFF' }}"
            >
        @endif
    @endif

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
