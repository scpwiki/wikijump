<?php

namespace Wikidot\DB;

/**
 * Object Model Class.
 *
 */
class OpenidEntry extends OpenidEntryBase
{

    public function getPageUnixName()
    {
        if ($this->getPageId()) {
            return PagePeer::instance()->selectByPrimaryKey($this->getPageId())->getUnixName();
        } else {
            return null;
        }
    }
}
