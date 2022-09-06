<?php

namespace Wikidot\DB;


/**
 * Object Model Class.
 *
 */
class ThemePeer extends ThemePeerBase
{
    public static function tempGet(): Theme
    {
        // TEMP: We're decommissioning this table, but for now we need it
        // So this function returns a hardcoded theme that looks fine for now
        return self::instance()->selectByPrimaryKey(20);
    }
}
