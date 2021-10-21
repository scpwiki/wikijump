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
            @if (empty($plain) || !$plain)
                {{-- TODO: Flag as objectionable functionality  --}}
                <a href="/flag">
                    {{ __('frame.footer.REPORT_FLAG') }}
                </a>
            @endif
        </div>
    </div>

    @if (empty($plain) || !$plain)
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
    @endif
</footer>
