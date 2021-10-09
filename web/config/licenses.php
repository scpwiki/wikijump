<?php
declare(strict_types=1);

use \Wikijump\Common\License;

/**
 * This is a list of licenses that administrators of your farm can pick for their wiki.
 *
 * Keep in mind:
 * - The order this array is in is the order shown in the UI.
 * - The ID of the license must remain constant because it is stored in the database.
 */

// TODO: add localization
return [
    // Creative Commons 4.0
    new License(
        'cc_by_sa_4',
        'Creative Commons Attribution-ShareAlike 4.0 License (recommended)',
        'https://creativecommons.org/licenses/by-sa/4.0/',
    ),
    new License(
        'cc_by_4',
        'Creative Commons Attribution 4.0 License',
        'https://creativecommons.org/licenses/by/4.0/',
    ),
    new License(
        'cc_by_nd_4',
        'Creative Commons Attribution-NoDerivs 4.0 License',
        'https://creativecommons.org/licenses/by-nd/4.0/',
    ),
    new License(
        'cc_by_nc_4',
        'Creative Commons Attribution-NonCommercial 4.0 License',
        'https://creativecommons.org/licenses/by-nc/4.0/',
    ),
    new License(
        'cc_by_nc_sa_4',
        'Creative Commons Attribution-NonCommercial-ShareAlike 4.0 License',
        'https://creativecommons.org/licenses/by-nc-sa/4.0/',
    ),
    new License(
        'cc_by_nc_nd_4',
        'Creative Commons Attribution-NonCommercial-NoDerivs 4.0 License',
        'https://creativecommons.org/licenses/by-nc-nd/4.0/',
    ),

    // Creative Commons 3.0
    new License(
        'cc_by_sa_3',
        'Creative Commons Attribution-ShareAlike 3.0 License',
        'https://creativecommons.org/licenses/by-sa/3.0/',
    ),
    new License(
        'cc_by_3',
        'Creative Commons Attribution 3.0 License',
        'https://creativecommons.org/licenses/by/3.0/',
    ),
    new License(
        'cc_by_nd_3',
        'Creative Commons Attribution-NoDerivs 3.0 License',
        'https://creativecommons.org/licenses/by-nd/3.0/',
    ),
    new License(
        'cc_by_nc_3',
        'Creative Commons Attribution-NonCommercial 3.0 License',
        'https://creativecommons.org/licenses/by-nc/3.0/',
    ),
    new License(
        'cc_by_nc_sa_3',
        'Creative Commons Attribution-NonCommercial-ShareAlike 3.0 License',
        'https://creativecommons.org/licenses/by-nc-sa/3.0/',
    ),
    new License(
        'cc_by_nc_nd_3',
        'Creative Commons Attribution-NonCommercial-NoDerivs 3.0 License',
        'https://creativecommons.org/licenses/by-nc-nd/3.0/',
    ),

    // Other Licenses
    new License(
        'cc0',
        'CC0 (Public Domain)',
        'https://creativecommons.org/publicdomain/zero/1.0/',
    ),
    new License(
        'gnu_fdl_13',
        'GNU Free Documentation License 1.3',
        'https://www.gnu.org/copyleft/fdl.html',
    ),

    // Fallback
    new License(
        'standard',
        'Standard copyright (not recommended)',
        null,
    )
];
