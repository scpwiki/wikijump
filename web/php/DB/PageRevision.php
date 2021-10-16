<?php

namespace Wikidot\DB;


use Illuminate\Support\Facades\Cache;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Ozone;
use Exception;
use Wikijump\Models\User;

/**
 * Object Model Class.
 *
 */
class PageRevision extends PageRevisionBase
{

    public function getSourceText()
    {
        assert(!$this->getDiffSource(), "Revision uses diff-based storage");

        $c = new Criteria();
        $c->add("source_id", $this->getSourceId());

        $source = PageSourcePeer::instance()->selectOne($c);
        return $source->getText();
    }

    public function resetFlags()
    {
        $this->setFlagText(false);
        $this->setFlagTitle(false);
        $this->setFlagRename(false);
        $this->setFlagNew(false);
        $this->setFlagFile(false);
        $this->setFlagNewSite(false);
    }

    public function getUser()
    {
        if ($this->getUserId() == User::ANONYMOUS_USER) {
            return null;
        }
        return User::find($this->getUserId());
    }

    public function getUserOrString()
    {
        $user = $this->getUser();
        if ($user == null) {
            return $this->getUserString();
        } else {
            return $user;
        }
    }

    public function getMetadata()
    {
        return PageMetadataPeer::instance()->selectByPrimaryKey($this->getMetadataId());
    }

    public function getPage()
    {
        if (is_array($this->prefetched)) {
            if (in_array('page', $this->prefetched)) {
                if (in_array('page', $this->prefetchedObjects)) {
                    return $this->prefetchedObjects['page'];
                } else {
                    $obj = new Page($this->sourceRow);
                    $obj->setNew(false);
                    $this->prefetchedObjects['page'] = $obj;
                    return $obj;
                }
            }
        }
        return PagePeer::instance()->selectByPrimaryKey($this->getPageId());
    }

    public function save()
    {
        try {
            $page = $this->getPage();
            if ($page) {
                $key = "sitechangesfeed..".$page->getSiteId();
                Cache::forget($key);

                $tkey = "siterevisions_lc..".$page->getSiteId();
                Cache::put($tkey, time(), 3600);

                $tkey = "pagerevisions_lc..".$page->getPageId();
                Cache::put($tkey, time(), 3600);
            }
        } catch (Exception $e) {
        }

        parent::save();
    }
}
