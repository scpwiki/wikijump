{{--
    View for navbar elements. See `frame.blade.php`.

    data:
        $items
--}}

<ul class="wj-sidebar-nav-elements">
    @foreach($items as $item_name => $item_links)
        <li>
            @if(is_array($item_links))
                <wj-sidebar-nav-dropdown class="wj-sidebar-nav-dropdown">
                    <details class="wj-sidebar-nav-dropdown-block">
                        <summary class="wj-sidebar-nav-dropdown-button">
                            <span class="wj-sidebar-nav-dropdown-button-text">
                                {{ $item_name }}
                            </span>
                            @include('next.components.sprite', ['sprite' => 'wj-downarrow'])
                        </summary>
                        <ul class="wj-sidebar-nav-dropdown-links">
                            @foreach($item_links as $link_name => $link_url)
                                <li>
                                    <a class="wj-sidebar-nav-dropdown-link"
                                       href="{{ $link_url }}"
                                       tabindex="-1"
                                    >
                                        {{ $link_name }}
                                    </a>
                                </li>
                            @endforeach
                        </ul>
                    </details>
                </wj-sidebar-nav-dropdown>
            @else
                <div class="wj-sidebar-nav-link">
                    <a href="{{ $item_links }}">{{ $item_name }}</a>
                </div>
            @endif
        </li>
    @endforeach
</ul>
