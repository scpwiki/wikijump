<?php

namespace Wikidot\DB;


use Illuminate\Support\Facades\Cache;
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
        Cache::forget($fkey);
        parent::save();
    }
}
