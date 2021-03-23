<?php

namespace Wikidot\DB;


/**
 * Object Model Class.
 *
 */
class FtsEntry extends FtsEntryBase
{

    public function getUrl()
    {
        if ($this->getPageId() !== null) {
            return '/'.$this->getUnixName();
        } else {
            return '/forum/t-'.$this->getThreadId().'/'.$this->getUnixName();
        }
    }
}
