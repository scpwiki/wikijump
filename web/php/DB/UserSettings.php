<?php

namespace Wikidot\DB;


/**
 * Object Model Class.
 *
 */
class UserSettings extends UserSettingsBase
{

    public function getReceivePm()
    {
        return trim(parent::getReceivePm());
    }
}
