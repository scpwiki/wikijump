<?php
namespace DB;

/**
 * Object Model class.
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
