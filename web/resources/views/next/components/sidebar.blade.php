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
        @isset($navbar_items)
            @include("next.components.sidebar-nav-elements", [
                'items' => $navbar_items,
            ])

            <hr class="wj-sidebar-nav-hr" />
        @endisset

        {!! $sidebar_content !!}
    </div>
</wj-sidebar>
