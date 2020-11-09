<?php
use DB\ForumCategoryPeer;

class ForumNewThreadModule extends SmartyModule
{

    private $category;
    protected $processPage = true;

    public function build($runData)
    {

        $site = $runData->getTemp("site");
        $pl = $runData->getParameterList();
        $categoryId = $pl->getParameterValue("c");

        if ($categoryId == null || !is_numeric($categoryId)) {
            throw new ProcessException(_("No forum category has been specified."));
        }

        // check for suggested title
        $sTitle = $pl->getParameterValue("title");

        $c = new Criteria();
        $c->add("category_id", $categoryId);
        $c->add("site_id", $site->getSiteId());

        $category = ForumCategoryPeer::instance()->selectOne($c);

        if ($category == null) {
            throw new ProcessException(_("No forum category has been specified."));
        }

        WDPermissionManager::instance()->hasForumPermission('new_thread', $runData->getUser(), $category);

        // keep the session - i.e. put an object into session storage not to delete it!!!
        $runData->sessionAdd("keep", true);

        $this->category = $category;
        $runData->contextAdd("category", $category);

        $userId = $runData->getUserId();
        if ($userId == null) {
            $userString = $runData->createIpString();
            $runData->contextAdd("anonymousString", $userString);
        }

        if ($sTitle) {
            $runData->contextAdd("title", $sTitle);
        }
    }

    public function processPage($out, $runData)
    {

        if ($this->category != null) {
            $out = preg_replace("/<title>(.+?)<\/title>/is", "<title>\\1 ".preg_quote_replacement(htmlspecialchars($this->category->getName()))."</title>", $out);
            $title = '<a href="/forum/c-'.$this->category->getCategoryId().'/'.htmlspecialchars($this->category->getUnixifiedName()).'">'.htmlspecialchars($this->category->getName()).'</a> / '._('new thread');

            $out = preg_replace('/<div id="page-title">(.*?)<\/div>/is', '<div id="page-title">'.preg_quote_replacement($title).'</div>', $out);
        }
        return $out;
    }
}
