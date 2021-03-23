<?php

namespace Wikidot\DB;


use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Ozone;
use Wikidot\Utils\ODiff;
use Exception;

/**
 * Object Model Class.
 *
 */
class PageRevision extends PageRevisionBase
{

    public function getSourceText()
    {

        if ($this->getDiffSource() == false) {
            $c = new Criteria();
            $c->add("source_id", $this->getSourceId());

            $source = PageSourcePeer::instance()->selectOne($c);
            return $source->getText();
        } else {
            // select last revisions and sources.
            $q = "SELECT page_source.* FROM page_source, page_revision WHERE " .
                    "page_revision.page_id =".$this->getPageId()." ".
                    "AND page_revision.revision_id <= ".$this->getRevisionId()." " .
                    "AND (page_revision.flag_text = TRUE OR page_revision.flag_new = TRUE) " .
                    "AND page_revision.source_id = page_source.source_id " .
                    "ORDER BY page_revision.revision_id DESC " .
                    "LIMIT ".($this->getSinceFullSource()+1);

            $c = new Criteria();
            $c->setExplicitQuery($q);
            $sources = PageSourcePeer::instance()->select($c);

            // original source...
            $s = end($sources);
            $s0 = $s->getText();

            $differ = new ODiff();
            while ($s = prev($sources)) {
                $s0 = $differ->patchString($s0, $s->getText());
                if ($differ->getErrors() != null) {
                    return "Error processing the source - please report the problem to the support";
                }
            }

            return trim($s0);
        }
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
        if ($this->getUserId() == 0) {
            return null;
        }
        if (is_array($this->prefetched)) {
            if (in_array('ozone_user', $this->prefetched)) {
                if (in_array('ozone_user', $this->prefetchedObjects)) {
                    return $this->prefetchedObjects['ozone_user'];
                } else {
                    $obj = new OzoneUser($this->sourceRow);
                    $obj->setNew(false);
                    $this->prefetchedObjects['ozone_user'] = $obj;
                    return $obj;
                }
            }
        }
        return OzoneUserPeer::instance()->selectByPrimaryKey($this->getUserId());
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

    public function getOzoneUser()
    {
        return $this->getUser();
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
                $mc = Ozone::$memcache;
                $mc->delete($key);

                $tkey = "siterevisions_lc..".$page->getSiteId();
                $mc->set($tkey, time(), 0, 3600);

                $tkey = "pagerevisions_lc..".$page->getPageId();
                $mc->set($tkey, time(), 0, 3600);
            }
        } catch (Exception $e) {
        }

        parent::save();
    }
}
