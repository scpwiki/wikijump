<?php
declare(strict_types=1);

// TODO: add localization
return [

    /**
     * Information on licenses supported by the platform.
     *
     * The order matters here: it's the order these are shown to the user.
     * The URL field may be null if there is no URL for this license.
     * Additionally, there are two optional fields:
     *  - 'label' => What is displayed in the selector list. Defaults to the name.
     *  - 'unless' => Whether to prepend the "Unless otherwise stated" comment.
     */

    'raw' => [
        // Creative Commons 4.0
        [
            'id' => 'cc_by_sa_4',
            'name' => 'Creative Commons Attribution-ShareAlike 4.0 License',
            'label' => 'Creative Commons Attribution-ShareAlike 4.0 License (recommended)',
            'url' => 'https://creativecommons.org/licenses/by-sa/4.0/',
        ],
        [
            'id' => 'cc_by_4',
            'name' => 'Creative Commons Attribution 4.0 License',
            'url' => 'https://creativecommons.org/licenses/by/4.0/',
        ],
        [
            'id' => 'cc_by_nd_4',
            'name' => 'Creative Commons Attribution-NoDerivs 4.0 License',
            'url' => 'https://creativecommons.org/licenses/by-nd/4.0/',
        ],
        [
            'id' => 'cc_by_nc_4',
            'name' => 'Creative Commons Attribution-NonCommercial 4.0 License',
            'url' => 'https://creativecommons.org/licenses/by-nc/4.0/',
        ],
        [
            'id' => 'cc_by_nc_sa_4',
            'name' => 'Creative Commons Attribution-NonCommercial-ShareAlike 4.0 License',
            'url' => 'https://creativecommons.org/licenses/by-nc-sa/4.0/',
        ],
        [
            'id' => 'cc_by_nc_nd_4',
            'name' => 'Creative Commons Attribution-NonCommercial-NoDerivs 4.0 License',
            'url' => 'https://creativecommons.org/licenses/by-nc-nd/4.0/',
        ],

        // Creative Commons 3.0
        [
            'id' => 'cc_by_sa_3',
            'name' => 'Creative Commons Attribution-ShareAlike 3.0 License',
            'url' => 'https://creativecommons.org/licenses/by-sa/3.0/',
        ],
        [
            'id' => 'cc_by_3',
            'name' => 'Creative Commons Attribution 3.0 License',
            'url' => 'https://creativecommons.org/licenses/by/3.0/',
        ],
        [
            'id' => 'cc_by_nd_3',
            'name' => 'Creative Commons Attribution-NoDerivs 3.0 License',
            'url' => 'https://creativecommons.org/licenses/by-nd/3.0/',
        ],
        [
            'id' => 'cc_by_nc_3',
            'name' => 'Creative Commons Attribution-NonCommercial 3.0 License',
            'url' => 'https://creativecommons.org/licenses/by-nc/3.0/',
        ],
        [
            'id' => 'cc_by_nc_sa_3',
            'name' => 'Creative Commons Attribution-NonCommercial-ShareAlike 3.0 License',
            'url' => 'https://creativecommons.org/licenses/by-nc-sa/3.0/',
        ],
        [
            'id' => 'cc_by_nc_nd_3',
            'name' => 'Creative Commons Attribution-NonCommercial-NoDerivs 3.0 License',
            'url' => 'https://creativecommons.org/licenses/by-nc-nd/3.0/',
        ],

        // Other Licenses
        [
            'id' => 'cc0',
            'name' => 'CC0 (Public Domain)',
            'url' => 'https://creativecommons.org/publicdomain/zero/1.0/',
        ],
        [
            'id' => 'gnu_fdl_13',
            'name' => 'GNU Free Documentation License 1.3',
            'url' => 'https://www.gnu.org/copyleft/fdl.html',
        ],

        // Fallback
        [
            'id' => 'standard',
            'name' => 'standard copyright',
            'label' => 'Standard copyright (not recommended)',
            'url' => null,
            'unless' => false,
        ],
    ]

];
