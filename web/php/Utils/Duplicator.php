<?php

namespace Wikidot\Utils;

use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Database\Database;
use Ozone\Framework\ODate;
use Wikidot\DB\AdminPeer;
use Wikidot\DB\Admin;
use Wikidot\DB\Member;
use Wikidot\DB\ThemePeer;
use Wikidot\DB\CategoryPeer;
use Wikidot\DB\PagePeer;
use Wikidot\DB\ForumGroupPeer;
use Wikidot\DB\ForumCategoryPeer;
use Wikidot\DB\FilePeer;
use Wikidot\DB\PageSource;
use Wikidot\DB\PageMetadata;
use Wikidot\DB\PageRevision;
use Wikidot\DB\Page;
use Wikidot\DB\PageCompiled;
use Wikidot\DB\PageTagPeer;

class Duplicator
{

    private $owner;
    private $excludedCategories = array();

    private $pageMap;

    public function cloneSite($site, $siteProperties, $attrs = array())
    {

        $db = Database::connection();
        $db->begin();
        /*
         * Hopefully attrs contains a set of parameters that determine
         * the behoviour of the duplicatior.
         */
        $nsite = clone ($site);
        $nsite->setNew(true);
        $nsite->setSiteId(null);

        $nsite->setUnixName($siteProperties['unixname']);
        if (isset($siteProperties['name'])) {
            $nsite->setName($siteProperties['name']);
        }
        if (isset($siteProperties['subtitle'])) {
            $nsite->setSubtitle($siteProperties['subtitle']);
        }
        if (isset($siteProperties['description'])) {
            $nsite->setDescription($siteProperties['description']);
        }
        if (array_key_exists('private', $siteProperties)) {
            if ($siteProperties['private']) {
                $nsite->setPrivate(true);
            } else {
                $nsite->setPrivate(false);
            }
        }
        $nsite->setCustomDomain(null);
        $nsite->save();

        /* Site settings. */
        $settings = $site->getSettings();
        $settings->setNew(true);
        $settings->setSiteId($nsite->getSiteId());
        $settings->save();

        /* Now handle site owner. */
        $c = new Criteria();
        $c->add('site_id', $site->getSiteId());
        $c->add('founder', true);
        $owner = AdminPeer::instance()->selectOne($c);

        $this->owner = $owner;

        $admin = new Admin();
        $admin->setSiteId($nsite->getSiteId());
        $admin->setUserId($owner->getUserId());
        $admin->setFounder(true); // will be nonremovable
        $admin->save();
        $member = new Member();
        $member->setSiteId($nsite->getSiteId());
        $member->setUserId($owner->getUserId());
        $member->setDateJoined(new ODate());
        $member->save();

        /* Theme(s). */
        $c = new Criteria();
        $c->add('site_id', $site->getSiteId());
        $themes = ThemePeer::instance()->select($c);
        $themeMap = array();
        $nthemes = array();
        foreach ($themes as $theme) {
            $ntheme = clone($theme);
            $ntheme->setNew(true);
            $ntheme->setSiteId($nsite->getSiteId());
            $ntheme->setThemeId(null);
            $ntheme->save();
            $themeMap[$theme->getThemeId()] = $ntheme->getThemeId();
            $nthemes[] = $ntheme;
        }
        foreach ($nthemes as $ntheme) {
            if ($ntheme->getExtendsThemeId() && isset($themeMap[$ntheme->getExtendsThemeId()])) {
                $ntheme->setExtendsThemeId($themeMap[$ntheme->getExtendsThemeId()]);
                $ntheme->save();
            }
        }


        // get all categories from the site
        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        $categories = CategoryPeer::instance()->select($c);

        foreach ($categories as $cat) {
            if (!in_array($cat->getName(), $this->excludedCategories)) {
                $ncategory = $this->duplicateCategory($cat, $nsite);
                /* Check if is using a custom theme. */
                if ($ncategory->getThemeId() && isset($themeMap[$ncategory->getThemeId()])) {
                    $ncategory->setThemeId($themeMap[$ncategory->getThemeId()]);
                    $ncategory->save();
                }
                if ($ncategory->getTemplateId()) {
                    $ncategory->setTemplateId($this->pageMap[$ncategory->getTemplateId()]);
                    $ncategory->save();
                }
            }
        }

        /* Recompile WHOLE site. */
        $od = new Outdater();
        $od->recompileWholeSite($nsite);

        /* Handle forum too. */

        $fs = $site->getForumSettings();
        if ($fs) {
            $fs->setNew(true);
            $fs->setSiteId($nsite->getSiteId());
            $fs->save();

            /* Copy existing structure. */
            $c = new Criteria();
            $c->add('site_id', $site->getSiteId());
            $groups = ForumGroupPeer::instance()->select($c);

            foreach ($groups as $group) {
                $ngroup = clone($group);
                $ngroup->setNew(true);
                $ngroup->setGroupId(null);
                $ngroup->setSiteId($nsite->getSiteId());
                $ngroup->save();

                $c = new Criteria();
                $c->add('group_id', $group->getGroupId());
                $categories = ForumCategoryPeer::instance()->select($c);
                foreach ($categories as $category) {
                    $ncategory = clone($category);
                    $ncategory->setNew(true);
                    $ncategory->setCategoryId(null);
                    $ncategory->setNumberPosts(0);
                    $ncategory->setNumberThreads(0);
                    $ncategory->setLastPostId(null);
                    $ncategory->setSiteId($nsite->getSiteId());
                    $ncategory->setGroupId($ngroup->getGroupId());
                    $ncategory->save();
                }
            }
        }

        /* Copy ALL files from the filesystem. */
        $srcDir = WIKIJUMP_ROOT."/web/files--sites/".$site->getUnixName();
        $destDir = WIKIJUMP_ROOT."/web/files--sites/".$nsite->getUnixName();

        $cmd = 'cp -r '. escapeshellarg($srcDir) . ' ' . escapeshellarg($destDir);
        exec($cmd);

        /* Copy file objects. */

        $c = new Criteria();
        $c->add('site_id', $site->getSiteId());
        $files = FilePeer::instance()->select($c);
        foreach ($files as $file) {
            $nfile = clone($file);
            $nfile->setSiteId($nsite->getSiteId());
            $nfile->setNew(true);
            $nfile->setFileId(null);
            $nfile->setSiteId($nsite->getSiteId());
            /* Map to a new page objects. */
            $pageId = $this->pageMap[$file->getPageId()];
            $nfile->setPageId($pageId);
            $nfile->save();
        }

        $db->commit();
        return $nsite;
    }

    /**
     * Duplicates the site by copying all the pages & categories & settings.
     */
    public function duplicateSite($site, $nsite)
    {
        $owner = $this->owner;
        // first copy settings

        // site_settings
        $settings = $site->getSettings();
        $settings->setNew(true);
        $settings->setSiteId($nsite->getSiteId());
        $settings->save();

        // add user as admin
        if ($owner) {
            $admin = new Admin();
            $admin->setSiteId($nsite->getSiteId());
            $admin->setUserId($owner->getUserId());
            $admin->setFounder(true); // will be nonremovable
            $admin->save();
            $member = new Member();
            $member->setSiteId($nsite->getSiteId());
            $member->setUserId($owner->getUserId());
            $member->setDateJoined(new ODate());
            $member->save();
        }

        // get all categories from the site
        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        $categories = CategoryPeer::instance()->select($c);

        foreach ($categories as $cat) {
            if (!in_array($cat->getName(), $this->excludedCategories)) {
                $this->duplicateCategory($cat, $nsite);
            }
        }

        // recompile WHOLE site!!!
        $od = new Outdater();
        $od->recompileWholeSite($nsite);
    }

    public function setOwner($user)
    {
        $this->owner = $user;
    }

    public function duplicateCategory($category, $nsite)
    {
        $cat = clone ($category);
        $cat->setNew(true);
        $cat->setCategoryId(null);
        $cat->setSiteId($nsite->getSiteId());
        $cat->save();
        // copy pages
        $c = new Criteria();
        $c->add("category_id", $category->getCategoryId());
        $pages = PagePeer::instance()->select($c);
        foreach ($pages as $page) {
            $this->duplicatePage($page, $nsite, $cat);
        }
        return $cat;
    }

    public function duplicatePage($page, $nsite, $ncategory, $newUnixName = null)
    {

        if ($newUnixName == null) {
            $newUnixName = $page->getUnixName();
        }

        // check if page exists - if so, forcibly delete!!!
        $p = PagePeer::instance()->selectByName($nsite->getSiteId(), $newUnixName);
        if ($p) {
            PagePeer::instance()->deleteByPrimaryKey($p->getPageId());
        }

        $owner = $this->owner;
        $now = new ODate();
        // create new page object based on the existing page
        $nsource = new PageSource();
        $nsource->setText($page->getSource());
        $nsource->save();

        $meta = $page->getMetadata();
        $nmeta = new PageMetadata();
        $nmeta->setTitle($meta->getTitle());
        $nmeta->setUnixName($newUnixName);
        if ($owner) {
            $nmeta->setOwnerUserId($owner->id);
        } else {
            $nmeta->setOwnerUserId($meta->getOwnerUserId());
        }
        $nmeta->save();

        $rev = $page->getCurrentRevision();
        $nrev = new PageRevision();
        $nrev->setSiteId($nsite->getSiteId());
        $nrev->setSourceId($nsource->getSourceId());
        $nrev->setMetadataId($nmeta->getMetadataId());
        $nrev->setFlagNew(true);
        $nrev->setFlagNewSite(true);
        $nrev->setDateLastEdited($now);
        $nrev->setUserId($owner->id);
        $nrev->obtainPK();

        $npage = new Page();
        $npage->setSiteId($nsite->getSiteId());
        $npage->setCategoryId($ncategory->getCategoryId());
        $npage->setRevisionId($nrev->getRevisionId());
        $npage->setSourceId($nsource->getSourceId());
        $npage->setMetadataId($nmeta->getMetadataId());
        $npage->setTitle($page->getTitle());
        $npage->setUnixName($newUnixName);
        $npage->setDateLastEdited($now);
        $npage->setDateCreated($now);
        $npage->setLastEditUserId($owner->id);
        $npage->setOwnerUserId($owner->id);

        $npage->save();
        $nrev->setPageId($npage->getPageId());
        $nrev->save();

        $ncomp = new PageCompiled();
        $ncomp->setPageId($npage->getPageId());
        $ncomp->setDateCompiled($now);
        $ncomp->save();

        /* Copy tags too. | NOTE This is very deprecated. Will revisit if we don't plan on deprecating this entire feature. —Yossi */
        $c = new Criteria();
        $c->add('page_id', $page->getPageId());
        $tags = PageTagPeer::instance()->select($c);
        foreach ($tags as $tag) {
            $tag->setNew(true);
            $tag->setTagId(null);
            $tag->setSiteId($nsite->getSiteId());
            $tag->setPageId($npage->getPageId());
            $tag->save();
        }

        $this->pageMap[$page->getPageId()] = $npage->getPageId();
    }

    public function addExcludedCategory($categoryName)
    {
        $this->excludedCategories[] = $categoryName;
    }

    /**
     * Dumps everything.
     */
    public function dumpSite($site)
    {

        $dump = array();
        $settings = $site->getSettings();
        $fs = $site->getForumSettings();

        $dump['settings'] = $settings;
        $dump['forumSettings'] = $fs;

        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        $categories = CategoryPeer::instance()->select($c);

        $dump['categories'] = $categories;

        $dump['pages'] = array();

        foreach ($categories as $cat) {
            $c = new Criteria();
            $c->add("category_id", $cat->getCategoryId());
            $pages = PagePeer::instance()->select($c);
            foreach ($pages as &$p) {
                $p->setTemp("source", $p->getSource());
                $p->setTemp("meta", $p->getMetadata());
            }
            $dump['pages'][$cat->getCategoryId()] = $pages;
        }

        return $dump;
    }

    public function restoreSite($nsite, $dump)
    {

        $settings = $dump['settings'];

        // site_settings
        $settings->setNew(true);
        $settings->setSiteId($nsite->getSiteId());
        $settings->save();

        $forumSettings = $dump['forumSettings'];
        $forumSettings->setNew(true);
        $forumSettings->setSiteId($nsite->getSiteId());
        $forumSettings->save();

        // add user as admin
        $owner = $this->owner;
        if ($owner) {
            $admin = new Admin();
            $admin->setSiteId($nsite->getSiteId());
            $admin->setUserId($owner->getUserId());
            $admin->setFounder(true); // will be nonremovable
            $admin->save();
            $member = new Member();
            $member->setSiteId($nsite->getSiteId());
            $member->setUserId($owner->getUserId());
            $member->setDateJoined(new ODate());
            $member->save();
        }

        $categories = $dump['categories'];

        foreach ($categories as $category) {
            $cat = clone ($category);
            $cat->setNew(true);
            $cat->setCategoryId(null);
            $cat->setSiteId($nsite->getSiteId());
            $cat->save();

            // get pages
            $pages = $dump['pages'][$category->getCategoryId()];

            foreach ($pages as $page) {
                $newUnixName = $page->getUnixName();

                $now = new ODate();
                // create new page object based on the existing page
                $nsource = new PageSource();
                $nsource->setText($page->getTemp("source"));
                $nsource->save();

                $meta = $page->getTemp("meta");
                $nmeta = new PageMetadata();
                $nmeta->setTitle($meta->getTitle());
                $nmeta->setUnixName($newUnixName);
                if ($owner) {
                    $nmeta->setOwnerUserId($owner->getUserId());
                } else {
                    $nmeta->setOwnerUserId($meta->getOwnerUserId());
                }
                $nmeta->save();

                $nrev = new PageRevision();
                $nrev->setSiteId($nsite->getSiteId());
                $nrev->setSourceId($nsource->getSourceId());
                $nrev->setMetadataId($nmeta->getMetadataId());
                $nrev->setFlagNew(true);
                $nrev->setDateLastEdited($now);
                $nrev->setUserId($owner->getUserId());
                $nrev->obtainPK();

                $npage = new Page();
                $npage->setSiteId($nsite->getSiteId());
                $npage->setCategoryId($cat->getCategoryId());
                $npage->setRevisionId($nrev->getRevisionId());
                $npage->setSourceId($nsource->getSourceId());
                $npage->setMetadataId($nmeta->getMetadataId());
                $npage->setTitle($page->getTitle());
                $npage->setUnixName($newUnixName);
                $npage->setDateLastEdited($now);
                $npage->setLastEditUserId($owner->getUserId());
                $npage->setOwnerUserId($owner->getUserId());

                $npage->save();
                $nrev->setPageId($npage->getPageId());
                $nrev->save();

                $ncomp = new PageCompiled();
                $ncomp->setPageId($npage->getPageId());
                $ncomp->setDateCompiled($now);
                $ncomp->save();
            }
        }

        $od = new Outdater();
        $od->recompileWholeSite($nsite);
    }
}
