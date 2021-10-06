{{-- -
  data:
    $social_url
    $social_url
    $social_title
    $social_description
    $social_image
    $social_twitter_handle

    $title
    $theme_color

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

    {{-- Security --}}
    <meta name="referrer" content="no-referrer">
    {{-- TODO:
    <meta
        http-equiv="Content-Security-Policy"
        content=""
    > --}}

    {{-- Preloads, Preconnects --}}
    {{-- TODO: figure out preloads
      <link rel="preload" href="{{ vite_asset('../css/base.scss') }}" as="style">
      <link rel="preload" href="{{ vite_asset('../scripts/index.ts') }}" as="script">
    - --}}
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
    <meta name="color-scheme" content="light dark">
    @isset($theme_color)
        <meta name="theme-color" content="{{ $theme_color }}">
        <meta name="msapplication-TileColor" content="{{ $theme_color }}">
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
    @isset($favicon_mask)
        @isset($theme_color)
            <link rel="mask-icon" href="{{ vite_asset($favicon_mask) }}" color="{{ $theme_color }}">
        @endisset
        @empty($theme_color)
            <link rel="mask-icon" href="{{ vite_asset($favicon_mask) }}" color="#FFF">
        @endempty
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
