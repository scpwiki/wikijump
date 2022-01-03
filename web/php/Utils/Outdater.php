<?php
declare(strict_types=1);

namespace Wikidot\Utils;

use Illuminate\Support\Facades\Cache;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Database\Database;
use Wikidot\DB\Page;
use Wikidot\DB\PagePeer;
use Wikidot\DB\CategoryPeer;
use Wikidot\DB\SitePeer;
use Wikijump\Models\PageContents;
use Wikijump\Services\Deepwell\DeepwellService;
use Wikijump\Services\Wikitext\Backlinks;
use Wikijump\Services\Wikitext\LegacyTemplateAssembler;
use Wikijump\Services\Wikitext\PageInfo;
use Wikijump\Services\Wikitext\ParseRenderMode;
use Wikijump\Services\Wikitext\WikitextBackend;

final class Outdater
{
    private static Outdater $instance;
    private array $vars = [];
    private Backlinks $link_stats;
    private int $recurrenceLevel = 0;

    public static function instance()
    {
        if (self::$instance == null) {
            self::$instance = new Outdater();
        }
        return self::$instance;
    }

    public function __construct($baseRecurrenceLevel = 0)
    {
        $this->recurrenceLevel = $baseRecurrenceLevel + 1;
    }

    public function pageEvent($eventType, Page $page, ?string $old_slug = null)
    {
        if ($this->recurrenceLevel > 5) {
            return;
        }

        switch ($eventType) {
            case 'new_page':
                $this->recompilePage($page);
                $this->updateLinks($page);
                $this->recompileIncludedByPage($page);
                $this->outdatePageCache($page);
                $this->handleNavigationElement($page);
                $this->handleTemplateChange($page);
                break;
            case 'source_changed':
                // NOTE: outgoing links only need to be updated
                $this->recompilePage($page);
                $this->outdatePageCache($page);
                $this->updateLinks($page);
                $this->recompileIncludedByPage($page);
                $this->handleNavigationElement($page);
                $this->handleTemplateChange($page);
                break;
            case 'title_changed':
                // NOTE: incoming links only need to be updated
                $this->recompilePage($page);
                $this->outdatePageCache($page);
                $this->updateLinks($page);
                $this->outdateDescendantsCache($page);
                $this->outdatePageTagsCache($page);
                break;
            case 'tag_change':
                $this->recompilePage($page);
                $this->outdatePageCache($page);
                $this->outdateDescendantsCache($page);
                $this->outdatePageTagsCache($page);
                break;
            case 'rename':
                // NOTE: incoming links only need to be updated
                $this->recompilePage($page);
                $this->updateLinks($page);
                $this->updateLinksMissing($page->getSiteId(), $old_slug);
                $this->recompileIncludedByPage($page);
                $this->recompileIncludedBySlug($old_slug);
                $this->outdateDescendantsCache($page);
                $this->outdatePageCache($old_slug);
                $this->outdatePageCache($page);
                $this->outdatePageTagsCache($page);
                $this->handleTemplateChange($page);
                $this->handleTemplateChange($old_slug);
                break;
            case 'delete':
                // Previously $page was the slug being deleted.
                // However, this is not great for typechecking, and
                // we lose some information we can take advantage of.
                //
                // So instead we pass in the slug to calls that want a string,
                // and the page when that information would be helpful.

                // NOTE: incoming links only need to be updated
                $this->updateLinksMissing($page->getSiteId(), $page->getUnixName());
                $this->recompileIncludedByPage($page);
                $this->outdatePageTagsCache($page->getUnixName());
                $this->outdatePageCache($page->getUnixName());
                $this->handleTemplateChange($page->getUnixName());
                break;
            case 'parent_changed':
                $this->outdatePageCache($page);
                $this->outdateDescendantsCache($page);
                break;
            case 'file_change':
                $this->recompilePage($page);
                $this->outdatePageCache($page);
                $this->recompileIncludedByPage($page);
                break;
            case 'page_vote':
                $this->outdatePageCache($page);
                $this->outdateRatingStars($page);
                break;
        }

        // reset vars
        $this->vars = [];
    }

    public function forumEvent($eventType, $parm = null)
    {
        switch ($eventType) {
            case 'post_save':
                // $parm is the post object here
                $this->handleForumPostSave($parm);
                break;
            case 'thread_save':
                // $parm is the post object here
                $this->handleForumThreadSave($parm);
                break;
            case 'outdate_forum':
                $this->handleWholeForumOutdate();
                break;
        }
    }

    public function categoryEvent($eventType, $category = null)
    {
        switch ($eventType) {
            case 'category_save':
                $this->outdateCategoryPagesCache($category);
                break;
        }
    }

    public function themeEvent($eventType, $theme = null)
    {
        switch ($eventType) {
            case 'theme_save':
                $this->outdateThemeDependentCategories($theme);
                break;
        }
    }

    public function siteEvent($eventType, $site = null)
    {
        switch ($eventType) {
            case 'sitewide_change':
                $this->outdateAllPagesCache($site);
                break;
            case 'delete':
                $this->handleSiteDelete($site);
                break;
        }
    }

    /**
     * This is the place where pages are compiled!
     *
     * @param Page $page
     */
    private function recompilePage(Page $page): void
    {
        // compiled content not up to date. recompile!
        $contents = PageContents::getLatestFull($page->getPageId());
        $wikitext = $contents->wikitext;

        /* Find out if the category is using any templates. */
        if (!preg_match('/(:|^)_/', $page->getUnixName())) {
            $category = $page->getCategory();
            $categoryName = $category->getName();
            $templatePage = PagePeer::instance()->selectByName(
                $page->getSiteId(),
                ($categoryName == '_default' ? '' : $categoryName.':') .'_template'
            );

            if ($templatePage) {
                $templateSource = $templatePage->getSource();
                $wikitext = LegacyTemplateAssembler::assembleTemplate($wikitext, $templateSource, $page);
            }
        }

        $pageInfo = PageInfo::fromPageObject($page);
        $wt = WikitextBackend::make(ParseRenderMode::PAGE, $pageInfo);
        $result = $wt->renderHtml($wikitext);
        $contents->compiled_html = $result->body;
        $contents->generator = $wt->version();
        $contents->save();
        $this->link_stats = $result->link_stats;
    }

    private function updateLinks(Page $page): void
    {
        DeepwellService::getInstance()->updateLinks($page->getSiteId(), $page->getPageId(), $this->link_stats);
    }

    private function updateLinksMissing(string $site_id, string $page_slug): void
    {
        DeepwellService::getInstance()->updateLinksMissing($site_id, $page_slug, $this->link_stats);
    }

    private function recompileIncludedByPage(Page $page): void
    {
        $connections = DeepwellService::getInstance()->getLinksTo($page->getSiteId(), $page->getPageId());
        $this->recompiledIncludedPagesInternal($connections);
    }

    private function recompileIncludedBySlug(string $site_id, string $slug): void
    {
        $connections = DeepwellService::getInstance()->getLinksToMissing($site_id, $slug);
        $this->recompiledIncludedPagesInternal($connections);
    }

    private function recompiledIncludedPagesInternal($connections): void
    {
        foreach ($connections as $connection) {
            $page = PagePeer::getInstance()->selectByPrimaryKey($connection->from_page_id);
            $outdater = new Outdater($this->recurrence_level);
            $outdater->pageEvent('source_changed', $page);
        }
    }

    public function outdateDescendantsCache($page)
    {
        // to keep breadcrumbs up-to-date
        //get all descendants.
        $rec = 0;

        $c = new Criteria();
        $c->add("parent_page_id", $page->getPageId());

        $pages = PagePeer::instance()->select($c);

        while ($pages !== null && count($pages)>0 && $rec<10) {
            $p2 = array();
            foreach ($pages as $p) {
                $this->outdatePageCache($p);
                $c = new Criteria();
                $c->add("parent_page_id", $p->getPageId());
                $ptmp = PagePeer::instance()->select($c);
                $p2 = array_merge($p2, $ptmp);
            }
            $pages = $p2;
            $rec++;
        }
    }

    public function outdatePageCache($page)
    {
        // both levels!
        $site = $GLOBALS['site'];
        $now = time();
        if (is_string($page)) {
            $pageName = $page;
        } else {
            $pageName = $page->getUnixName();
        }

        $key = 'url..'.$site->getUnixName() . '.' . GlobalProperties::$URL_DOMAIN . '/'.$pageName;
        $cd = $site->getCustomDomain();
        if ($cd !== null && $cd !=='') {
            $key = 'url..'.$cd.'/'.$pageName;
            Cache::forget($key);
        }

        // check if default landing page
        if ($site->getDefaultPage() == $pageName) {
            $key = 'url..'.$site->getUnixName(). '.' . GlobalProperties::$URL_DOMAIN;
            if ($cd !== null && $cd !=='') {
                $key = 'url..'.$cd;
                Cache::forget($key);
            }
        }

        Cache::forget($key);
        $key = 'page..'.$site->getUnixName().'..'.$pageName;
        Cache::forget($key);

        /* Touch the catefory "last change" timestamp. */

        if (strpos($pageName, ":") != false) {
            $tmp0 = explode(':', $pageName);
            $categoryName = $tmp0[0];
        } else {
            $categoryName = "_default";
        }
        $ckey = 'pagecategory_lc..'.$site->getUnixName().'..'.$categoryName;
        Cache::put($ckey, $now, 10000);

        $ckey = 'pageall_lc..'.$site->getUnixName();
        Cache::put($ckey, $now, 10000);
        /*
         * Outdate code blocks.
         */

        $ckey = 'pagecodeblocks..' . $site->getSiteId() . '..' . $pageName;
        Cache::forget($ckey);
    }

    /**
     * Check if this page is a navigation page for any of the categories.
     * If so - clear cache of all pages in the category.
     */
    private function handleNavigationElement($page)
    {
        // get default cat
        $site = $GLOBALS['site'];
        $pUnixName = $page->getUnixName();
        $dcat = CategoryPeer::instance()->selectByName('_default', $site->getSiteId());

        $q = "SELECT unix_name FROM page WHERE category_id IN ( " .
                "SELECT category_id FROM category WHERE nav_default = false " .
                    "AND (top_bar_page_name='$pUnixName' OR side_bar_page_name='$pUnixName') " .
                    "AND site_id='".$site->getSiteId()."'";
        if ($dcat->getTopBarPageName() === $pUnixName || $dcat->getSideBarPageName() === $pUnixName) {
            $q .= "UNION SELECT category_id FROM category WHERE nav_default = true " .
                    "AND site_id='".$site->getSiteId()."'";
        }
        $q .= ")";

        $db = Database::connection();
        $r = $db->query($q);
        while ($row = $r->nextRow()) {
            $name = $row['unix_name'];
            $this->outdatePageCache($name);
        }
    }

    private function outdateCategoryPagesCache($category, $site = null)
    {
        if (!$site) {
            $site = SitePeer::instance()->selectByPrimaryKey($category->getSiteId());
        }

        $q = "SELECT unix_name FROM page WHERE category_id='".$category->getCategoryId()."'";
        $db = Database::connection();
        $r = $db->query($q);
        while ($row = $r->nextRow()) {
            $name = $row['unix_name'];
        }

        // the above is not necesarily necessary. try the below code:
        $aKey = 'category_lc..'.$site->getUnixName().'..'.$category->getName();
        $now = time();
        Cache::put($aKey, $now, 7200);
        $key = 'category..'.$site->getSiteId().'..'.$category->getName();
        Cache::forget($key);
        $key = 'categorybyid..'.$site->getSiteId().'..'.$category->getCategoryId();
        Cache::forget($key);
    }

    private function outdateThemeDependentCategories($theme)
    {

        $c = new Criteria();
        $c->add("theme_id", $theme->getThemeId());
        $cats = CategoryPeer::instance()->select($c);
        foreach ($cats as $cat) {
            $this->outdateCategoryPagesCache($cat);
        }
    }

    private function outdateAllPagesCache($site)
    {
        if (!$site) {
            $site = $GLOBALS['site'];
        }
        $q = "SELECT unix_name FROM page WHERE site_id='".$site->getSiteId()."'";

        $db = Database::connection();
        $r = $db->query($q);
        while ($row = $r->nextRow()) {
            $name = $row['unix_name'];
            $this->outdatePageCache($name);
        }
        // again the above is not necesarily necessary. try the below code:
        $q = "SELECT name FROM category WHERE site_id='".$site->getSiteId()."'";

        $db = Database::connection();
        $r = $db->query($q);
        $now = time();
        while ($row = $r->nextRow()) {
            $name = $row['name'];
            $aKey = 'category_lc..'.$site->getUnixName().'..'.$name;
            Cache::put($aKey, $now, 7200);
        }
    }

    public function handleForumPostSave($post)
    {
        // create an antry with mod time
        $now = time();
        $site = $GLOBALS['site'];

        // outdate forum thread
        $tkey = 'forumthread_lc..'.$site->getUnixName().'..'.$post->getThreadId();
        Cache::put($tkey, $now, 1000);

        // outdate forum category
        $thread = $post->getForumThread();
        $tkey = 'forumcategory_lc..'.$site->getUnixName().'..'.$thread->getCategoryId();
        Cache::put($tkey, $now, 1000);

        // outdate whole forum (affects the main view)

        $tkey = 'forumstart_lc..'.$site->getUnixName();
        Cache::put($tkey, $now, 1000);

        // check if forum not related to any page (page discussion)
        if ($thread->getPageId() !== null) {
            $page = PagePeer::instance()->selectByPrimaryKey($thread->getPageId());
            $this->outdatePageCache($page);
        }
    }

    public function handleForumThreadSave($thread)
    {
        // create an entry with mod time
        $now = time();
        $site = $GLOBALS['site'];

        // outdate forum thread
        $tkey = 'forumthread_lc..'.$site->getUnixName().'..'.$thread->getThreadId();
        Cache::put($tkey, $now, 1000);

        // outdate forum category
        $tkey = 'forumcategory_lc..'.$site->getUnixName().'..'.$thread->getCategoryId();
        Cache::put($tkey, $now, 1000);

        // outdate whole forum (affects the main view)
        $tkey = 'forumstart_lc..'.$site->getUnixName();
        Cache::put($tkey, $now, 1000);
    }

    private function handleWholeForumOutdate()
    {
        // create an antry with mod time
        $now = time();
        $site = $GLOBALS['site'];

        $key = 'forumall_lc..'.$site->getUnixName();
        Cache::put($key, $now, 3600);
    }

    public function recompileCategory($category)
    {
        $site = SitePeer::instance()->selectByPrimaryKey($category->getSiteId());
        $site0 = $GLOBALS['site'];
        $GLOBALS['site'] = $site;
        $c = new Criteria();
        $c->add("category_id", $category->getCategoryId());
        $pages = PagePeer::instance()->select($c);

        foreach ($pages as $page) {
            $this->recompilePage($page);
            $this->outdatePageCache($page);
            $this->updateLinks($page);
        }

        $GLOBALS['site']=$site0;
    }

    public function recompileWholeSite($site)
    {
        $site0 = $GLOBALS['site'];
        $GLOBALS['site'] = $site;
        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        $pages = PagePeer::instance()->select($c);

        foreach ($pages as $page) {
            $this->recompilePage($page);
            $this->outdatePageCache($page);
            $this->updateLinks($page);
        }

        $GLOBALS['site']=$site0;
    }

    public function outdatePageTagsCache($page)
    {
        if (is_string($page)) {
            return;
        } else {
            $siteId = $page->getSiteId();
        }

        $key = "page_tags_lc..".$siteId;

        Cache::put($key, time(), 3600);
    }

    public function outdateRatingStars($page)
    {
        $siteId = $page->getSiteId();

        $key = "top_rated_pages_lc..".$siteId;

        Cache::put($key, time(), 3600);
    }

    private function handleSiteDelete($site)
    {
        $key = "sitesettings..".$site->getSiteId();
        Cache::forget($key);

        $key = 'site..'.$site->getUnixName();
        Cache::forget($key);

        $key = 'site_cd..'.$site->getCustomDomain();
        Cache::forget($key);
    }

    private function handleTemplateChange($page)
    {
        if (is_string($page)) {
            if (strpos($page, ":") != false) {
                $tmp0 = explode(':', $page);
                $categoryName = $tmp0[0];
            } else {
                $categoryName = "_default";
            }
            if (preg_match('/_template$/', $page)) {
                $site = $GLOBALS['site'];
                $category = CategoryPeer::instance()->selectByName($categoryName, $site->getSiteId(), false);
                $this->recompileCategory($category);
            }
        } elseif (preg_match('/_template$/', $page->getUnixName())) {
            $category = $page->getCategory();
            $this->recompileCategory($category);
        }
    }
}
