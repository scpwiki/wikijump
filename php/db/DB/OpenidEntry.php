<?php
namespace DB;

/**
 * Object Model class.
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
