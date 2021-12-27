{{--
    Generic navbar found on most pages.

    The `$navbar_items` variable needs to have the following structure:
        $navbar_items = [
            'dropdown-name' => [
                'link-name' => 'link-url',
                ...
            ],
            // or
            'link-name' => 'link-url',
            ...
        ];

    data:
        $sidebar_content (determines if a sidebar reveal button should be visible)
        $navbar_items
 --}}

{{-- TODO: Page search widget --}}
{{-- TODO: Locale selector--}}
{{-- TODO: Dark/light mode selector --}}
<nav id="navbar" class="dark" aria-label="{{ __('navigation') }}">
    @includeWhen(
        isset($sidebar_content) || isset($navbar_items),
        'next.components.sidebar-button'
    )

    <wj-component-loader ld-load="Search" ld-skeleton="block">
    </wj-component-loader>

    @includeWhen(isset($navbar_items), 'next.components.navbar-elements', [
        'items' => $navbar_items,
    ])

    {{-- only gets displayed on small screens --}}
    <wj-component-loader ld-load="ClientStatus" background="false">
    </wj-component-loader>
</nav>
