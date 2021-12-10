{{--
    data:
        $show_subscribed
        $unsubscribe_url

    sections:
        content
--}}
-- {{ $SERVICE_NAME }} --
-- {{$HTTP_SCHEMA}}://{{$URL_HOST}} --

@yield("content")
@if(isset($show_subscribed) && $show_subscribed && isset($unsubscribe_url))

-------------------------------------------------------------------------------

{{ __('email.SUBSCRIBED') }}
{{ __('email.UNSUBSCRIBE_TEXT') }}
{{ $unsubscribe_url }}
@endif
