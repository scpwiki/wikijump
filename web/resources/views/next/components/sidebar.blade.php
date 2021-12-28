{{--
    data:
        $navbar_items
        $sidebar_content (UNESCAPED)
--}}

<wj-sidebar id="sidebar"
            role="complementary"
            aria-label="{{ __('sidebar') }}"
>
    <div id="sidebar_sticky" role="presentation">
        {{-- only shows up on small screens --}}
        <div id="sidebar_close">
            <wj-sidebar-close-button id="sidebar_close_button"
                    type="button"
                    aria-label="{{ __('close') }}"
                    aria-controls="sidebar"
            >
                @include("next.components.sprite", [ "sprite" => "wj-close" ])
            </wj-sidebar-close-button>
        </div>


        @isset($navbar_items)
            @include("next.components.sidebar-nav-elements", [
                'items' => $navbar_items,
            ])

            <hr class="wj-sidebar-nav-hr" />
        @endisset

        {!! $sidebar_content !!}
    </div>
</wj-sidebar>
