<?php

namespace Wikidot\Utils;

use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Database\Database;
use Wikidot\DB\SitePeer;
use Wikidot\DB\PagePeer;
use Wikidot\DB\CategoryPeer;

class Deleter
{
    private static $instance;

    private $vars = array();

    private $recurrenceLevel = 0;

    public static function instance()
    {
        if (self::$instance == null) {
            self::$instance = new Deleter();
        }
        return self::$instance;
    }

    public function deletePage($page, $site = null)
    {
        // I hope everything here is within a TRANSACTION!!!

        if (!$page) {
            return;
        }

        if (!$site) {
            $site = SitePeer::instance()->selectByPrimaryKey($page->getSiteId());
        }

        // delete the sources and metadatas

        $db = Database::connection();

        // get descandants first
        $rec = 0;

        $c = new Criteria();
        $c->add("parent_page_id", $page->getPageId());

        $pages = PagePeer::instance()->select($c);

        // ok, these are direct children. need to clear the perent_page_id field

        $descs = array();
        while ($pages !== null && count($pages) > 0 && $rec < 10) {
            $p2 = array();
            foreach ($pages as $p) {
                $c = new Criteria();
                $c->add("parent_page_id", $p->getPageId());
                $ptmp = PagePeer::instance()->select($c);
                $p2 = array_merge($p2, $ptmp);

                if ($rec === 0) {
                    $p->setParentPageId(null);
                    $p->save();
                    // clear metadata
                    $m = $p->getMetadata();
                    $m->setParentPageId(null);
                    $m->save();
                }
            }
            $descs = array_merge($descs, $pages, $p2);
            $pages = $p2;
            $rec++;
        }

        $category = $page->getCategory();

        // sources
        $q = "DELETE FROM page_contents WHERE revision_id IN (SELECT page_revision.revision_id FROM page_revision WHERE page_id='" . $page->getPageId() . "')";
        $db->query($q);
        //metadatas
        $q = "DELETE FROM page_metadata WHERE metadata_id IN (SELECT page_revision.metadata_id FROM page_revision WHERE page_id='" . $page->getPageId() . "')";
        $db->query($q);
        // delete the page too
        $q = "DELETE FROM page WHERE page_id='" . $page->getPageId() . "'";
        $db->query($q);

        // remove from cache too.
        $outdater = new Outdater();
        $outdater->pageEvent('delete', $page);

        // outdate descs too
        foreach ($descs as $desc) {
            $outdater->outdatePageCache($desc);
        }

        // delete the category if empty
        if ($category->getName() != "_default") {
            $c = new Criteria();
            $c->add("category_id", $category->getCategoryId());
            $count = PagePeer::instance()->selectCount($c);

            if ($count == 0) {
                // delete the category
                CategoryPeer::instance()->delete($c);
                $outdater->categoryEvent('delete', $category, $site);
            }
        }

        // remove FILES (if any)
        $path = WIKIJUMP_ROOT . "/web/files--sites/" . $site->getSlug() . "/files/" . $page->getUnixName();
        exec('rm -r ' . escapeshellarg($path) . ' &> /dev/null');

    //
    }

    public function deleteSite($site)
    {
        if (!$site) {
            return;
        }
        // get all pages and delete each one

        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());

        $pages = PagePeer::instance()->select($c);

        foreach ($pages as $page) {
            $this->deletePage($page);
        }

        // delete forum? no. will be autodeleted based on the database constrains.

        // need to delete post revisions
        $db = Database::connection();
        $q = "DELETE FROM forum_post_revision WHERE forum_post_id IN (SELECT post_id FROM forum_post WHERE site_id= {$site->getSiteId()}";
        $db->query($q);
        //delete the site itself

        $outdater = new Outdater();
        $outdater->siteEvent('delete', $site);

        SitePeer::instance()->deleteByPrimaryKey($site->getSiteId());
    }
}
