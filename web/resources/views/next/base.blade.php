{{-- -
  data:
    $social_url
    $social_url
    $social_title
    $social_description
    $social_image
    $social_twitter_handle

    $title
    $color_scheme

    $favicon_svg
    $favicon_png
    $favicon_apple
    $favicon_mask

  sections:
    content

  stacks:
    preloads
    styles
    head
    scripts
- --}}

<!DOCTYPE html>
<html lang="{{ str_replace('_', '-', app()->getLocale()) }}">

<head>
    {{-- Functionality --}}
    <meta charset="utf-8">
    <base href="/">
    <meta name="viewport" content="width=device-width,initial-scale=1">

    {{-- Preloads, Preconnects --}}
    <link rel="preload" href="{{ vite_asset('../css/base.scss') }}" as="style">
    <link rel="preload" href="{{ vite_asset('../scripts/index.ts') }}" as="script">
    @stack("preloads")

    {{-- Social --}}
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
        <meta property="og:image" content="{{ vite_asset($social_image) }}">
    @endisset
    @isset($social_twitter_handle)
        <meta name="twitter:card" content="summary">
        <meta name="twitter:site" content="{{ $social_twitter_handle }}">
        <meta name="twitter:creator" content="{{ $social_twitter_handle }}">
    @endisset

    {{-- Browser Metadata --}}
    @isset($title)
        <title>{{ $title }}</title>
    @endisset
    <meta name="color-scheme" content="dark light">
    @isset($color_scheme)
        <meta name="theme-color" content="{{ $color_scheme }}">
        <meta name="msapplication-TileColor" content="{{ $color_scheme }}">
    @endisset

    {{-- Icons --}}
    @isset($favicon_svg)
        <link rel="icon" type="image/svg+xml" href="{{ vite_asset($favicon_svg) }}">
    @endisset
    @isset($favicon_png)
        <link rel="icon" type="image/png" href="{{ vite_asset($favicon_png) }}">
    @endisset
    @isset($favicon_apple)
        <link rel="apple-touch-icon" href="{{ vite_asset($favicon_apple) }}">
    @endisset
    @isset($favicon_mask && $color_scheme)
        <link rel="mask-icon" href="{{ vite_asset($favicon_mask) }}" color="{{ $color_scheme }}">
    @endisset

    {{-- Styles --}}
    @vite("resources/css/base.scss")
    @stack("styles")

    {{-- Vite --}}
    @client

    {{-- Misc --}}
    <link rel="author" href="humans.txt"> {{-- TODO --}}

    @stack("head")
</head>

<body class="light codetheme-dark">
    @yield("content")

    <div id="toasts"></div>
    <div id="modals"></div>

    {{-- Scripts --}}
    {{-- TODO: see if it's possible to make scripts load async --}}
    @vite("resources/scripts/index.ts")
    @stack("scripts")
</body>

</html>
