{{--
    View for navigation dropdowns. See `frame.blade.php`.

    data:
        $items
--}}

<ul class="wj-navbar-dropdowns">
    @foreach($items as $item_name => $item_links)
        <li>
            <wj-nav-dropdown class="wj-navbar-dropdown">
                <details class="wj-navbar-dropdown-block">
                    <summary class="wj-navbar-dropdown-button">
                        {{ $item_name }}
                        @include('next.components.ui-svg', ['sprite' => 'wj-downarrow'])
                    </summary>
                    <ul class="wj-navbar-dropdown-links">
                        @foreach($item_links as $link_name => $link_url)
                            <li>
                                <a class="wj-navbar-dropdown-link" href="{{ $link_url }}">
                                    {{ $link_name }}
                                </a>
                            </li>
                        @endforeach
                    </ul>
                </details>
            </wj-nav-dropdown>
        </li>
    @endforeach
</ul>
