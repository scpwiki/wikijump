{{--
    View for navbar elements. See `frame.blade.php`.

    data:
        $items
--}}

<ul class="wj-navbar-elements">
    @foreach($items as $item_name => $item_links)
        <li>
            @if(is_array($item_links))
                <wj-navbar-dropdown class="wj-navbar-dropdown">
                    <details class="wj-navbar-dropdown-block">
                        <summary class="wj-navbar-dropdown-button">
                            <span class="wj-navbar-dropdown-button-text">
                                {{ $item_name }}
                            </span>
                            @include('next.components.ui-svg', ['sprite' => 'wj-downarrow'])
                        </summary>
                        <ul class="wj-navbar-dropdown-links">
                            @foreach($item_links as $link_name => $link_url)
                                <li>
                                    <a class="wj-navbar-dropdown-link"
                                       href="{{ $link_url }}"
                                       tabindex="-1"
                                    >
                                        {{ $link_name }}
                                    </a>
                                </li>
                            @endforeach
                        </ul>
                    </details>
                </wj-navbar-dropdown>
            @else
                <div class="wj-navbar-link">
                    <a href="{{ $item_links }}">{{ $item_name }}</a>
                </div>
            @endif
        </li>
    @endforeach
</ul>
