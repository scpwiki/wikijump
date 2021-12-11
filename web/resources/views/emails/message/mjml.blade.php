{{--
    data:
        $level (unused)
        $subject (unused)
        $greeting
        $salutation
        $introLines
        $outroLines
        $actionText
        $actionUrl
        $displayableActionUrl

    Uses the same data as MailMessage.
--}}

@extends('emails.base-mjml', [
    'title' => $subject,
    'preview' => $greeting ?? ($introLines ? $introLines[0] : null),
])

@section('content')
    {{-- Greetings --}}
    @isset($greeting)
        <mj-section background-color="#fff" padding="0">
            <mj-column>
                <mj-text align="center">
                    <h1 style="padding-bottom: 0;">
                        {{ $greeting }}
                    </h1>
                </mj-text>
            </mj-column>
        </mj-section>
    @endisset

    {{-- Intro Lines --}}
    @isset($introLines)
        <mj-section background-color="#fff" padding="0 20px">
            <mj-column>
                @foreach ($introLines as $line)
                    <mj-text align="center" font-size="14px">{{ $line }}</mj-text>
                @endforeach
            </mj-column>
        </mj-section>
    @endisset

    {{-- Action Button --}}
    {{-- TODO: set color of button depending on $level --}}
    @isset($actionText, $actionUrl)
        <mj-section background-color="#fff" padding="5px 20px">
            <mj-column>
                <mj-button href="{{ $actionUrl }}"
                           background-color="#3869D4"
                           inner-padding="10px 45px"
                           font-size="16px"
                >
                    {{ $actionText }}
                </mj-button>
            </mj-column>
        </mj-section>
    @endisset

    {{-- Outro Lines --}}
    @isset($outroLines)
        <mj-section background-color="#fff" padding="20px 20px 0 20px">
            <mj-column>
                @foreach ($outroLines as $line)
                    <mj-text align="center" font-size="12px">{{ $line }}</mj-text>
                @endforeach
            </mj-column>
        </mj-section>
    @endisset

    {{-- Salutation --}}
    @isset($salutation)
        <mj-section background-color="#fff" padding="0 20px">
            <mj-column>
                <mj-text font-size="14px">{{ $salutation }}</mj-text>
            </mj-column>
        </mj-section>
    @endisset

    {{-- Subcopy --}}
    @isset($actionText, $actionUrl)
        <mj-section background-color="#fff" padding-bottom="0">
            <mj-column>
                <mj-divider border-width="2px"
                            border-color="#eee"
                            padding-top="0"
                            padding-bottom="0"
                />
            </mj-column>
        </mj-section>

        <mj-section background-color="#fff">
            <mj-column>
                <mj-text align="center" font-size="10px">
                    {{ __('email.SUBCOPY', ['action' => $actiontext]) }}
                    <br />
                    <br />
                    <a href="{{ $actionUrl }}">
                        {{ isset($displayableActionUrl) ? $displayableActionUrl : $actionUrl }}
                    </a>
                </mj-text>
            </mj-column>
        </mj-section>
    @endisset
@endsection
