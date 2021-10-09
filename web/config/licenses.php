<?php
declare(strict_types=1);

use \Wikijump\Common\License;

/**
 * This is a list of licenses that administrators of your farm can pick for their wiki.
 *
 * Keep in mind: the order this array is in is the order shown in the UI.
 */

// TODO: add localization
return [
    // Creative Commons 4.0
    new License(
        'Creative Commons Attribution-ShareAlike 4.0 License (recommended)',
        'https://creativecommons.org/licenses/by-sa/4.0/',
    ),
    new License(
        'Creative Commons Attribution 4.0 License',
        'https://creativecommons.org/licenses/by/4.0/',
    ),
    new License(
        'Creative Commons Attribution-NoDerivs 4.0 License',
        'https://creativecommons.org/licenses/by-nd/4.0/',
    ),
    new License(
        'Creative Commons Attribution-NonCommercial 4.0 License',
        'https://creativecommons.org/licenses/by-nc/4.0/',
    ),
    new License(
        'Creative Commons Attribution-NonCommercial-ShareAlike 4.0 License',
        'https://creativecommons.org/licenses/by-nc-sa/4.0/',
    ),
    new License(
        'Creative Commons Attribution-NonCommercial-NoDerivs 4.0 License',
        'https://creativecommons.org/licenses/by-nc-nd/4.0/',
    ),

    // Creative Commons 3.0
    new License(
        'Creative Commons Attribution-ShareAlike 3.0 License',
        'https://creativecommons.org/licenses/by-sa/3.0/',
    ),
    new License(
        'Creative Commons Attribution 3.0 License',
        'https://creativecommons.org/licenses/by/3.0/',
    ),
    new License(
        'Creative Commons Attribution-NoDerivs 3.0 License',
        'https://creativecommons.org/licenses/by-nd/3.0/',
    ),
    new License(
        'Creative Commons Attribution-NonCommercial 3.0 License',
        'https://creativecommons.org/licenses/by-nc/3.0/',
    ),
    new License(
        'Creative Commons Attribution-NonCommercial-ShareAlike 3.0 License',
        'https://creativecommons.org/licenses/by-nc-sa/3.0/',
    ),
    new License(
        'Creative Commons Attribution-NonCommercial-NoDerivs 3.0 License',
        'https://creativecommons.org/licenses/by-nc-nd/3.0/',
    ),

    // Other Licenses
    new License(
        'CC0 (Public Domain)',
        'https://creativecommons.org/publicdomain/zero/1.0/',
    ),
    new License(
        'GNU Free Documentation License 1.3',
        'https://www.gnu.org/copyleft/fdl.html',
    ),

    // Fallback
    new License(
        'Standard copyright (not recommended)',
        null,
    )
];
