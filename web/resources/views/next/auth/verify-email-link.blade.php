{{--
    View shown to people who have clicked on an email confirmation link.
    This page requires human interactivity before a POST request for
    confirming the email is sent.
--}}

<html>
    <head>
        <meta charset="utf-8">
        <meta name="viewport" content="width=device-width,initial-scale=1">

        <meta name="referrer" content="no-referrer">
        <meta name="csrf-token" content="{{ csrf_token() }}" />

        <title>{{ __('wiki-auth-verify-email-link') }}</title>

        @inline('resources/css/pages/verify-email-link.scss')
        @inline('resources/lib/verify-email-link.ts')

        @client
    </head>
    <body>
        <div id="verify_waiting">
            <h1>{{ __('wiki-auth-verify-email-link.waiting') }}</h1>
        </div>

        <div id="verify_please_interact" style="display: none;">
            <div id="verify_robot">🤖</div>
            <h1>{{ __('wiki-auth-verify-email-link.please-interact') }}</h1>
            <p>{{ __('wiki-auth-verify-email-link.instructions') }}</p>
        </div>

        <div id="verify_success" style="display: none;">
            <h1>{{ __('wiki-auth-verify-email-link.success') }}</h1>
        </div>

        <div id="verify_failure" style="display: none;">
            <h1>{{ __('error-api.internal') }}</h1>
        </div>
    </body>
</html>
