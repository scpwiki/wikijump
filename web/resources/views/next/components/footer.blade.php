{{--
    Footer for most pages.

    data:
        $plain (set to true to remove certain page-related elements)
        $license (see `config/licenses.php` and `App/Services/License`)
 --}}

<footer id="footer" aria-label="{{ __('frame.aria.FOOTER') }}">
    <div id="footer_main">
        <div id="footer_services">
            @if ($SERVICE_NAME != "")
                <a class="footer-services-partof"
                    href="{{$HTTP_SCHEMA}}://{{$URL_HOST}}"
                >
                    {{ __('footer-part-of', ['name' => $SERVICE_NAME]) }}
                </a>
                <span class="footer-services-sep">&#8212;</span>
            @endif
            <a class="footer-services-poweredby"
                href="https://github.com/scpwiki/wikijump"
            >
                {{ __('footer-powered-by') }}
            </a>
            <span class="footer-services-sep">&#8212;</span>
            <a href="{{ route("terms") }}">{{ __('terms') }}</a>
            <a href="{{ route("privacy") }}">{{ __('privacy') }}</a>
            <a href="{{ route("docs") }}">{{ __('docs') }}</a>
            <a href="{{ route("security") }}">{{ __('security') }}</a>
        </div>

        <div id="footer_actions">
            <a href="https://scuttle.atlassian.net/servicedesk/customer/portal/2">
                {{ __('footer-menu.report-bug') }}
            </a>
            @if (empty($plain) || !$plain)
                {{-- TODO: Flag as objectionable functionality  --}}
                <a href="{{ route("report-flag") }}">
                    {{ __('footer-menu.report-flag') }}
                </a>
            @endif
        </div>
    </div>

    @if (empty($plain) || !$plain)
        @isset($license)
            <div id="footer_license" aria-label="{{ __('license') }}">
                <a href="{{ $license->url() }}">
                    @if ($license->unless())
                        {{ __('footer-license-unless', ['license' => $license->name()]) }}
                    @else
                        {{  __('footer-license', ['license' => $license->name()]) }}
                    @endif
                </a>
            </div>
        @endisset
    @endif
</footer>
