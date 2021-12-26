{{--
    data:
        $sidebar_content (UNESCAPED)
--}}

<wj-sidebar id="sidebar"
            role="complementary"
            aria-label="{{ __('sidebar') }}"
>
    <div id="sidebar_sticky" role="presentation">
        {!! $sidebar_content !!}
    </div>
</wj-sidebar>
