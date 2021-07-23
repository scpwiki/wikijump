<?php

namespace Wikidot\Modules\Forum;

use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Ozone;
use Ozone\Framework\SmartyModule;
use Wikidot\DB\ForumCategoryPeer;
use Wikidot\DB\ForumThreadPeer;
use Wikidot\DB\FrontForumFeed;
use Wikidot\DB\FrontForumFeedPeer;
use Wikidot\DB\SitePeer;
use Wikidot\Utils\ProcessException;
use Wikidot\Utils\WDRenderUtils;
use Wikidot\Utils\WDStringUtils;
use Wikijump\Services\Wikitext\ParseRenderMode;
use Wikijump\Services\Wikitext\WikitextBackend;

class FrontForumModule extends SmartyModule
{

    protected $processPage = true;
    private $vars = array();

    public function render($runData)
    {
        $site = $runData->getTemp("site");
        $pl = $runData->getParameterList();
        $categoryIds = $pl->getParameterValue("category");

        $parmHash = md5(serialize($pl->asArray()));

        $key = 'frontforum_v..'.$site->getUnixName().'..'.$categoryIds.'..'.$parmHash;

        $valid = true;

        $mc = OZONE::$memcache;
        $struct = $mc->get($key);
        if (!$struct) {
            $valid = false;
        }
        $cacheTimestamp = $struct['timestamp'];

        $now = time();

        // now check lc for ALL categories involved
        $cats = preg_split('/[,;] ?/', $categoryIds);

        foreach ($cats as $cat) {
            $tkey = 'forumcategory_lc..'.$site->getUnixName().'..'.$cat; // last change timestamp
            $changeTimestamp = $mc->get($tkey);
            if ($changeTimestamp && $cacheTimestamp && $changeTimestamp <= $cacheTimestamp) {
                //cache valid
            } else {
                $valid = false;
                if (!$changeTimestamp) {
                    //  put timestamp
                    $mc->set($tkey, $now, 0, 864000);
                    $valid = false;
                }
            }
        }
        $akey = 'forumall_lc..'.$site->getUnixName();
        $allForumTimestamp = $mc->get($akey);
        if ($allForumTimestamp &&  $allForumTimestamp <= $cacheTimestamp) {
            //cache valid
        } else {
                $valid = false;
        }

        if (!$allForumTimestamp) {
            $mc->set($akey, $now, 0, 864000);
        }

        if ($valid) {
            $this->vars = $struct['vars'];
            return $struct['content'];
        }

        $out = parent::render($runData);

        // and store the data now
        $struct = array();
        $now = time();
        $struct['timestamp'] = $now;
        $struct['content'] = $out;
        $struct['vars']=$this->vars;

        $mc->set($key, $struct, 0, 864000);

        return $out;
    }

    public function build($runData)
    {
        $pl = $runData->getParameterList();
        $site = $runData->getTemp("site");
        $categoryIds = $pl->getParameterValue("category");

        $limit =  $pl->getParameterValue("limit");
        $offset =  $pl->getParameterValue("offset");

        if ($limit == null || !is_numeric($limit) || $limit > 20) {
            $limit = 20;
        }

        if ($categoryIds === null) {
            throw new ProcessException(_('No forum category has been specified. Please use attribute category="id" where id is the index number of the category.'), "no_category");
        }

        if (strlen($categoryIds)>90) {
            throw new ProcessException(_("Category string too long."), "max_categories");
        }

        $cats = preg_split('/[,;] ?/', $categoryIds);

        $ccat = new Criteria();
        $categories = array();

        if (count($cats)>20) {
            throw new ProcessException(_("Maximum number of categories exceeded."), "max_categories");
        }

        foreach ($cats as $categoryId) {
            if ($categoryId === null || !is_numeric($categoryId)) {
                throw new ProcessException(_('Problem parsing attribute "category".'), "no_category");
            }

            $category = ForumCategoryPeer::instance()->selectByPrimaryKey($categoryId);

            if ($category == null) {
                throw new ProcessException(_('Requested forum category does not exist.'), "no_category");
            }
            if ($category->getSiteId() !== $site->getSiteId()) {
                $fSite = SitePeer::instance()->selectByPrimaryKey($category->getSiteId());
                if ($fSite->getPrivate()) {
                    throw new ProcessException(_('The requested category belongs to a private site.'), "no_category");
                }
            }
            $category->setTemp("group", $category->getForumGroup());
            $categories[$category->getCategoryId()] = $category;
            $ccat->addOr("category_id", $category->getCategoryId());
        }
        $c = new Criteria();
        $c->addCriteriaAnd($ccat);

        $c->addOrderDescending("thread_id");
        $c->setLimit($limit, $offset);
        $threads = ForumThreadPeer::instance()->select($c);

        $format = $pl->getParameterValue("module_body");

        if ($format == null || $format == '') {
            $format = "" .
                    "+ %%linked_title%%\n\n" .
                    _("by")." %%author%% %%date|%O ago (%e %b %Y, %H:%M %Z)%%\n\n" .
                    "%%content%%\n\n%%comments%% | "._("category").": %%category%%";
        }

        // process the format and create the message template
        $wt = WikitextBackend::make(ParseRenderMode::FEED, null);
        $template = $wt->renderHtml($format)->body;

        $template = preg_replace(
            '/
            <p\s*>\s*
            (%%(
                (?:short)
                |(?:description)
                |(?:summary)
                |(?:content)
                |(?:long)
                |(?:body)
                |(?:text)
            )%%)
            \s*<\/\s*p>
            /smix',
            "<div>\\1</div>",
            $template
        );

        $items = array();

        foreach ($threads as $thread) {
            $post = $thread->getFirstPost();
            if (!$post) {
                continue;
            }

            $b = $template;
            $b = str_ireplace("%%title%%", htmlspecialchars($thread->getTitle()), $b);

            $b = preg_replace("/%%((linked_title)|(title_linked))%%/i", preg_quote_replacement('<a href="/forum/t-'.$thread->getThreadId().'/'.$thread->getUnixifiedTitle().'">'. htmlspecialchars($thread->getTitle()).'</a>'), $b);

            $b = str_ireplace("%%author%%", WDRenderUtils::renderUser($thread->getUserOrString(), array("image"=>true)), $b);

            $dateString = '<span class="odate">'.$thread->getDateStarted()->getTimestamp().'|%e %b %Y, %H:%M %Z|agohover</span>';
            $b = str_ireplace('%%date%%', $dateString, $b);
            $b = preg_replace('/%%date\|(.*?)%%/i', '<span class="odate">'.preg_quote_replacement($thread->getDateStarted()->getTimestamp()).'|\\1</span>', $b);

            $b = str_ireplace("%%comments%%", '<a href="/forum/t-'.$thread->getThreadId().'/'.$thread->getUnixifiedTitle().'">'._('Comments').': '.($thread->getNumberPosts()-1).'</a>', $b);
            $b = str_ireplace("%%link%%", '/forum/t-'.$thread->getThreadId().'/'.$thread->getUnixifiedTitle(), $b);
            $category = $categories[$thread->getCategoryId()];
            $b = str_ireplace("%%category%%", '<a href="/forum/c-'.$category->getCategoryId().'/'.$category->getUnixifiedName().'">'.htmlspecialchars($category->getTemp("group")->getName()." / ".$category->getName()).'</a>', $b);

            $b = preg_replace("/%%((description)|(short)|(summary))%%/i", preg_quote_replacement(htmlspecialchars($thread->getDescription())), $b);
            $b = preg_replace("/%%((body)|(text)|(long)|(content))%%/i", preg_quote_replacement($post->getText()), $b);
            $items[] = $b;
        }

        $runData->contextAdd("items", $items);

        // post a feed???

        $flabel = WDStringUtils::toUnixName($pl->getParameterValue("feed"));
        $page = $runData->getTemp("page");
        if ($flabel && $page) {
            $ftitle = trim($pl->getParameterValue("feedTitle"));
            if ($ftitle == '') {
                $ftitle = $site->getName()." feed";
            }
            $fdescription = $pl->getParameterValue("feedDescription");
            $fcats = trim($categoryIds);
            $parmhash = crc32($ftitle." ".$fcats);

            // first check the memcache!!! to avoid db connection.

            // get the feed object
            $c = new Criteria();
            $c->add("page_id", $page->getPageId());
            $c->add("label", $flabel);

            $feed = FrontForumFeedPeer::instance()->selectOne($c);
            if ($feed == null) {
                // create the feed
                $feed = new FrontForumFeed();
                $feed->setLabel($flabel);
                $feed->setTitle($ftitle);
                $feed->setCategories($fcats);
                $feed->setPageId($page->getPageId());
                $feed->setDescription($fdescription);
                $feed->setSiteId($site->getSiteId());
                $feed->save();
            } else {
                //  check hash
                if ($feed->getParmhash() != $parmhash) {
                    $feed->setTitle($ftitle);
                    $feed->setCategories($fcats);
                    $feed->setDescription($fdescription);
                    $feed->save();
                }
            }

            // and the feed url is:
            $feedUrl = "/feed/front/".$page->getUnixName()."/".$flabel.".xml";
            $this->vars['feedUrl'] = $feedUrl;
            $this->vars['feedTitle'] = $ftitle;
            $this->vars['feedLabel'] = $flabel;

            // put a link into text
            $runData->contextAdd("feedUri", $feedUrl);
        }
    }

    public function processPage($out, $runData)
    {

        if ($this->vars['feedUrl']) {
            $out = preg_replace(
                "/<\/head>/",
                '<link rel="alternate" type="application/rss+xml" title="'.preg_quote_replacement(htmlspecialchars($this->vars['feedTitle'])).'" href="'.$this->vars['feedUrl'].'"/></head>',
                $out,
                1
            );
        }
        return $out;
    }
}
