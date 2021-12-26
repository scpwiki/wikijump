{{--
    data:
        $title
        $preview
        $logo_src
        $show_subscribed
        $unsubscribe_url
        $social_links

    sections:
        content

    All content sections should have their background-color set to #fff.
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
        <mj-section background-color="#fff" padding-bottom="0">
            <mj-column>
                <mj-image width="400px"
                          src="{{ $HTTP_SCHEMA }}://{{ $URL_HOST }}/files--static/media/logo.png"
                          href="{{$HTTP_SCHEMA}}://{{$URL_HOST}}"
                          alt="{{ __('goto-service') }}"
                />
                <mj-divider border-width="2px"
                            border-color="#eee"
                            padding-bottom="0"
                            padding-top="20px"
                />
            </mj-column>
        </mj-section>

        {{-- Content --}}
        @yield('content')

        {{-- Social --}}
        @if(isset($social_links) && count($social_links) > 0)
            <mj-section padding-bottom="5px">
                <mj-column>
                    <mj-social text-padding="0 20px 0 0">
                        @foreach($social_links as $sl)
                            {{-- This is a bit messy, but Blade doesn't
                                 really offer a good way to do this --}}
                            <mj-social-element
                                {!! isset($sl['name']) ? 'name="' . $sl['name'] . '"' : '' !!}
                                {!! isset($sl['url']) ? 'href="' . $sl['url'] . '"' : '' !!}
                                {!! isset($sl['src']) ? 'src="' . $sl['src'] . '"' : '' !!}
                            >
                                {{ $sl['text'] ?? '' }}
                            </mj-social-element>
                        @endforeach
                    </mj-social>
                </mj-column>
            </mj-section>
        @endif

        {{-- Subscription --}}
        @if(isset($show_subscribed) && $show_subscribed && isset($unsubscribe_url))
            <mj-section padding-top="0">
                <mj-column>
                    <mj-text align="center" font-size="10px" line-height="12px">
                        {{ __('emails-subscribed') }}
                        <br />
                        <br />
                        <a href="{{ $unsubscribe_url }}">
                            {{ __('emails-unsubscribe') }}
                        </a>
                    </mj-text>
                </mj-column>
            </mj-section>
        @endif
    </mj-body>
</mjml>
