<?php
declare(strict_types=1);

namespace Wikijump\Models;

use Wikijump\Common\Enum;

/**
 * What kind of connection this is.
 *
 * This is not a Model, but is located in this namespace
 * because it is used by PageConnection and PageConnectionMissing.
 */
final class PageConnectionType extends Enum
{
    const INCLUDE_MESSY = 'include-messy';
    const INCLUDE_ELEMENTS = 'include-elements';
    const COMPONENT = 'component';
    const LINK = 'link';
}
