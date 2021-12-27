{{--
    Component for displaying an element from the `ui.svg` spritesheet.

    data:
        $sprite
--}}

@php
    $viewbox = '';
    switch ($sprite) {
        case 'wj-karma': $viewbox = '0 0 64 114' ; break;
        default:         $viewbox = '0 0 24 24'  ; break;
    }
@endphp

<svg class="wj-ui-sprite sprite-{{ $sprite }}"
     viewBox="{{ $viewbox }}"
     aria-hidden="true"
>
    <use href="/files--static/media/ui.svg#{{ $sprite }}"></use>
</svg>
