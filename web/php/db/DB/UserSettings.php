<?php
namespace DB;

/**
 * Object Model class.
 *
 */
class UserSettings extends UserSettingsBase
{

    public function getReceivePm()
    {
        return trim(parent::getReceivePm());
    }
}
