<?php
use DB\PagePeer;

class ForumCommentsModule extends SmartyModule
{

    protected $processPage = true;

    public function build($runData)
    {
        $pl = $runData->getParameterList();

        // check if forum is activated
        $site = $runData->getTemp("site");
        $fsettings = $site->getForumSettings();

        if (!$fsettings) {
            throw new ProcessException(_("Forum must be activated for the Comments module to work. Please use the Site Manager."));
        }

        $show = true;
        if ($pl->getParameterValue("hide", "MODULE")) {
            $show = false;
        }
        // but can be forced via uri parameter "comments"
        if ($pl->getParameterValue("comments") == "show") {
            $show = true;
        }

        $pageName = $runData->getTemp("pageUnixName");
        if ($pageName == null) {
            $show=false;
        }

        $title = $pl->getParameterValue("title", "MODULE");
        if ($title === null) {
            $title = _('Comments');
        }

        $runData->contextAdd("title", $title);

        $runData->contextAdd("showComments", $show);
    }

    public function processPage($out, $runData)
    {
        $site = $runData->getTemp("site");
        $pageName = $runData->getTemp("pageUnixName");
        if ($pageName == null) {
            return $out;
        }
        $page = PagePeer::instance()->selectByName($site->getSiteId(), $pageName);
        $pageId = $page->getPageId();
        $link = '/feed/page/comments-'.$pageId.'.xml';
        $title =  "Comments for the page \"".$page->getTitleOrUnixName()."\"";
        $out = preg_replace(
            "/<\/head>/",
            '<link rel="alternate" type="application/rss+xml" title="'.htmlspecialchars($title).'" href="'.$link.'"/></head>',
            $out,
            1
        );

        return $out;
    }
}
