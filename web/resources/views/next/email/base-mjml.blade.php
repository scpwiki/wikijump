{{--
    data:
        $title
        $preview
        $logo_src (https://i.imgur.com/pa0YEu2.png)
        $logo_href (https://www.wikijump.dev)
        $show_subscribed
        $unsubscribe_url

    sections:
        content
--}}

<mjml>
    <mj-head>
        @isset($title)
            <mj-title>{{ $title }}</mj-title>
        @endisset

        @isset($preview)
            <mj-preview>{{ $preview }}</mj-preview>
        @endisset
    </mj-head>

    <mj-body background-color="#eee">
        {{-- Space --}}
        <mj-section>
            <mj-column></mj-column>
        </mj-section>

        {{-- Header --}}
        <mj-section background-color="#fff">
            <mj-column>
                <mj-image width="400px"
                          src="{{ $logo_src }}"
                          href="{{ $logo_href }}"
                          alt="{{ __('email.GOTO_SITE') }}"
                />
                <mj-divider border-width="2px"
                            border-color="#aaa"
                            padding-bottom="0"
                            padding-top="20px"
                />
            </mj-column>
        </mj-section>

        {{-- Body --}}
        <mj-section background-color="#fff" padding-top="10px">
            <mj-column>
                @yield('content')
            </mj-column>
        </mj-section>

        {{-- Social --}}
        <mj-section padding-bottom="5px">
            <mj-column>
                <mj-social text-padding="0 10px 0 0">
                    <mj-social-element name="web"
                                       href="https://wikijump.org"
                    >
                        {{ __('email.social.BLOG') }}
                    </mj-social-element>

                    <mj-social-element name="twitter"
                                       href="https://twitter.com/getwikijump"
                    >
                        {{ __('email.social.TWITTER') }}
                    </mj-social-element>

                    <mj-social-element name="github"
                                       href="https://github.com/scpwiki/wikijump"
                    >
                        {{ __('email.social.GITHUB') }}
                    </mj-social-element>
                </mj-social>
            </mj-column>
        </mj-section>

        {{-- Subscription --}}
        @if(isset($show_subscribed) && $show_subscribed && isset($unsubscribe_url))
            <mj-section padding-top="0">
                <mj-column>
                    <mj-text align="center" font-size="10px" line-height="12px">
                        {{ __('email.SUBSCRIBED') }}
                        <br />
                        <a href="{{ $unsubscribe_url }}">
                            {{ __('email.UNSUBSCRIBE') }}
                        </a>
                    </mj-text>
                </mj-column>
            </mj-section>
        @endif
    </mj-body>
</mjml>
