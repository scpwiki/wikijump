<?php
declare(strict_types=1);

namespace Wikidot\Utils;

use Illuminate\Database\Eloquent\Collection;
use Illuminate\Support\Facades\Cache;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Database\Database;
use Ozone\Framework\ODate;
use Wikidot\DB\Page;
use Wikidot\DB\PagePeer;
use Wikidot\DB\CategoryPeer;
use Wikidot\DB\SitePeer;
use Wikijump\Models\PageConnection;
use Wikijump\Models\PageConnectionMissing;
use Wikijump\Models\PageConnectionType;
use Wikijump\Models\PageLink;
use Wikijump\Services\Wikitext\LegacyTemplateAssembler;
use Wikijump\Services\Wikitext\PageInfo;
use Wikijump\Services\Wikitext\ParseRenderMode;
use Wikijump\Services\Wikitext\WikitextBackend;

final class Outdater
{
    private static Outdater $instance;
    private array $vars = [];
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
                $this->fixInLinks($page);
                $this->fixOutLinks($page);
                $this->fixInclusions($page);
                $this->recompileIncludedByPage($page);
                $this->outdatePageCache($page);
                $this->handleNavigationElement($page);
                $this->handleTemplateChange($page);
                break;
            case 'source_changed':
                $this->recompilePage($page);
                $this->outdatePageCache($page);
                $this->fixOutLinks($page);
                $this->fixInclusions($page);
                $this->recompileIncludedByPage($page);
                $this->handleNavigationElement($page);
                $this->handleTemplateChange($page);
                break;
            case 'title_changed':
            case 'tag_change':
                $this->recompilePage($page);
                $this->outdatePageCache($page);
                $this->outdateDescendantsCache($page);
                $this->fixInLinks($page); // if dynamical link text = page title
                $this->outdatePageTagsCache($page);
                break;
            case 'rename':
                $this->recompilePage($page);
                $this->fixInLinks($page);
                $this->fixInLinks($old_slug);
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

                $this->fixInLinks($page->getUnixName());
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

        // TODO just save as $this->link_stats
        $this->vars['internal_links_present'] = $result->link_stats->internal_links_present;
        $this->vars['internal_links_absent'] = $result->link_stats->internal_links_absent;
        $this->vars['inclusions_present'] = $result->link_stats->inclusions_present;
        $this->vars['inclusions_absent'] = $result->link_stats->inclusions_absent;
        $this->vars['external_links'] = $result->link_stats->external_links;
    }

    /**
     * Recompile pages which point to this page (either real or wanted).
     *
     * @param $page Page|string The page to check incoming links for.
     */
    private function fixInLinks($page)
    {
        if (is_string($page)) {
            // Wanted page, doesn't exist
            $this->fixInLinksAbsent($page);
        } else {
            // Real page, exists
            $this->fixInLinksPresent($page);
        }
    }

    private function fixInLinksPresent(Page $page)
    {
        $links = PageConnection::where('from_page_id', $page->getPageId());
        foreach ($links as $link) {
            $page = PagePeer::instance()->selectByPrimaryKey($link->from_page_id);
            $outdater = new Outdater($this->recurrenceLevel);
            $outdater->pageEvent('source_changed', $page);
        }
    }

    private function fixInLinksAbsent(string $page_name)
    {
        $links = PageConnectionMissing::where('from_page_name', $page_name);
        foreach ($links as $link) {
            $page = PagePeer::instance()->selectByPrimaryKey($link->from_page_id);
            $outdater = new Outdater($this->recurrenceLevel);
            $outdater->pageEvent('source_changed', $page);
        }
    }

    /**
     * Update the list of links that originate from this page.
     */
    private function fixOutLinks(Page $page): void
    {
        $this->fixConnectionsPresent($page, $this->vars['internal_links_present'], PageConnectionType::LINK);
        $this->fixConnectionsAbsent($page, $this->vars['internal_links_absent'], PageConnectionType::LINK);
        $this->fixOutLinksExternal($page, $this->vars['external_links']);
    }

    /**
     * Update the list of pages that include this one.
     */
    private function fixInclusions(Page $page): void
    {
        $this->fixConnectionsPresent($page, $this->vars['inclusions_present'], PageConnectionType::INCLUDE_MESSY);
        $this->fixConnectionsAbsent($page, $this->vars['inclusions_absent'], PageConnectionType::INCLUDE_MESSY);
    }

    /**
     * A generic helper function for adjusting an outdated page connections
     * table in light of page updates.
     *
     * @param Page $page The page connections are being adjusted for.
     * @param array $items_present The new list of items to be preserved as connections.
     * @param string $connection_type The PageConnectionType enum value to store in the table.
     */
    private function fixConnectionsPresent(Page $page, array $items_present, string $connection_type): void
    {
        /*
         * Turns a list of individual items into a mapping of those items
         * and the number of items found in the list.
         *
         * For instance, ['a', 'a', 'b', 'c', 'a', 'c']
         * becomes ['a' => 3, 'b' => 1, 'c' => 2].
         */
        $items_to_add = array_count_values($items_present);

        /*
         * Find existing links in the database.
         *
         * For each link in the database, either it will be in
         * $links_present or it won't.
         *
         * If it exists, then we don't need to insert it, so we
         * should remove from $links_resent.
         *
         * If it doesn't, then this was formerly a link, but isn't
         * anymore, so we need to delete it.
         */
        PageConnection::where([
            'from_page_id' => $page->getPageId(),
            'from_site_id' => $page->getSiteId(),
            'connection_type' => $connection_type,
        ])->chunk(100, function ($connections) use (&$items_to_add) {
            $this->updateItemsCounts($connections, 'to_page_id', $items_to_add);
        });

        /*
         * Now that we have all the links to add, we iterate
         * over them and insert them into the database.
         */
        foreach ($items_to_add as $item_page_id => $count) {
            // TODO retrieve site_id along with page_id to avoid this query
            $item_site_id = PagePeer::instance()->selectByPrimaryKey($item_page_id)->getSiteId();

            PageConnection::create([
                'from_page_id' => $page->getPageId(),
                'from_site_id' => $page->getSiteId(),
                'to_page_id' => $item_page_id,
                'to_site_id' => $item_site_id,
                'connection_type' => $connection_type,
                'count' => $count,
            ]);
        }
    }

    /**
     * A generic helper function for adjusting an outdated missing page connections
     * table in light of page updates.
     *
     * @param Page $page The page connections are being adjusted for.
     * @param array $items_absent The new list of items to be preserved as missing connections.
     * @param string $connection_type The PageConnectionType enum value to store in the table.
     */
    private function fixConnectionsAbsent(Page $page, array $items_absent, string $connection_type): void
    {
        $items_to_add = array_count_values($items_absent);

        /*
         * Similar to fixConnectionsPresent, this finds existing links
         * in the database, removes duplicates, and then saves
         * the link additions / removals based on the new list.
         */
        PageConnectionMissing::where([
            'from_page_id' => $page->getPageId(),
            'from_site_id' => $page->getSiteId(),
            'connection_type' => $connection_type,
        ])->chunk(100, function ($connections) use (&$items_to_add) {
            $this->updateItemsCounts($connections, 'to_page_id', $items_to_add);
        });

        foreach ($items_to_add as $item_page_name => $count) {
            // TODO where does the site name come from?
            // we will probably need to rethink link_stats along
            // with the previous todo
            $item_site_name = null;

            PageConnectionMissing::create([
                'from_page_id' => $page->getPageId(),
                'from_site_id' => $page->getSiteId(),
                'to_page_name' => $item_page_name,
                'to_site_name' => $item_site_name,
                'connection_type' => $connection_type,
                'count' => $count,
            ]);
        }

    }

    private function fixOutLinksExternal(Page $page, array $links_external): void
    {
        $links_to_add = array_count_values($links_external);

        /*
         * Again similar to fixOutLinksPresent and fixOutLinksAbsent,
         * see those methods for this same pattern.
         */
        PageLink::where([
            'page_id' => $page->getPageId(),
            'site_id' => $page->getSiteId(),
        ])->chunk(100, function ($links) use (&$links_to_add) {
            $this->updateItemsCounts($links, 'page_id', $links_to_add);
        });

        foreach ($links_to_add as $url => $count) {
            PageLink::create([
                'page_id' => $page->getPageId(),
                'site_id' => $page->getSiteId(),
                'url' => $url,
                'count' => $count,
            ]);
        }
    }

    private function updateItemsCounts(Collection $db_items, string $page_id_field, array &$items_to_add): void
    {
        foreach ($db_items as $db_item) {
            $page_id = $db_item->$page_id_field;
            $count = $items_to_add[$page_id];

            if (isset($count)) {
                // We just need to update, not insert
                unset($items_to_add[$page_id]);

                $db_item->count = $count;
                $db_item->save();
            } else {
                // No remaining items, remove it
                $db_item->delete();
            }
        }
    }

    private function recompileIncludedByPage(Page $page): void
    {
        PageConnection::where([
            'to_page_id' => $page->getPageId(),
            'to_site_id' => $page->getSiteId(),
            'connection_type' => PageConnectionType::INCLUDE_MESSY,
        ])->chunk(100, function ($connections) {
            $this->recompiledIncludedPagesBatch($connections);
        });
    }

    private function recompileIncludedBySlug(string $slug): void
    {
        PageConnectionMissing::where([
            'to_page_name' => $slug,
            'connection_type' => PageConnectionType::INCLUDE_MESSY,
        ])->chunk(100, function ($connections) {
            $this->recompiledIncludedPagesBatch($connections);
        });
    }

    private function recompiledIncludedPagesBatch($connections): void
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
            $this->fixOutLinks($page);
            $this->fixInclusions($page);
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
            $this->fixOutLinks($page);
            $this->fixInclusions($page);
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
