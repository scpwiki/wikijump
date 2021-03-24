<?php

namespace Wikidot\DB;


use Ozone\Framework\Ozone;

/**
 * Object Model Class.
 *
 */
class FrontForumFeed extends FrontForumFeedBase
{

    public function save()
    {
        // set parmhash
        $this->setParmhash(crc32($this->getTitle()." ".$this->getCategories()));
        $page = PagePeer::instance()->selectByPrimaryKey($this->getPageId());
        $site = $GLOBALS['site'];
        $fkey = "frontforumfeedobject..".$site->getUnixName().'..'.$page->getUnixName().'..'.$this->getLabel();
        $mc = Ozone::$memcache;
        $mc->delete($fkey);
        parent::save();
    }
}
