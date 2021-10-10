{{--
    Component for displaying an element from the `ui.svg` spritesheet.

    data:
        $sprite
--}}

@php
    // TODO: figure out if there is some automatic way to do this via styling
    $viewbox = '';
    switch ($sprite) {
        case 'wj-karma':             $viewbox = '0 0 64 114' ; break;
        case 'wj-toggle':            $viewbox = '0 0 128 64' ; break;
        case 'wj-checkbox':          $viewbox = '0 0 64 64'  ; break;
        case 'wj-clipboard-copy':    $viewbox = '0 0 24 24'  ; break;
        case 'wj-clipboard-success': $viewbox = '0 0 24 24'  ; break;
        case 'wj-downarrow':         $viewbox = '0 0 26 18'  ; break;
    }
@endphp

<svg class="wj-ui-sprite sprite-{{ $sprite }}" viewBox="{{ $viewbox }}">
    <use href="/wikijump--next/assets/media/ui.svg#{{ $sprite }}"></use>
</svg>
