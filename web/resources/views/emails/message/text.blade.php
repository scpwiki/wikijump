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
        $displayableActionUrl (unused)

    Uses the same data as MailMessage.
--}}
@extends('emails.base-text')
@section('content')
@isset($greeting)
{{ $greeting }}
@endisset

@foreach($introLines as $line)
{{ $line }}
@endforeach

@isset($actionText, $actionUrl)
{{ $actionText }}: {!! $actionUrl !!}
@endisset

@foreach($outroLines as $line)
{{ $line }}
@endforeach

@isset($salutation)
{{ $salutation }}
@endisset
@endsection
