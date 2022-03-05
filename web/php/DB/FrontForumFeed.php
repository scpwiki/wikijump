<?php

namespace Wikidot\DB;

use Illuminate\Support\Facades\Cache;
use Wikijump\Services\Deepwell\Models\Page;

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
        $page = Page::findIdOnly($this->getPageId());
        $site = $GLOBALS['site'];
        $fkey = "frontforumfeedobject..".$site->getSlug().'..'.$page->getUnixName().'..'.$this->getLabel();
        Cache::forget($fkey);
        parent::save();
    }
}
