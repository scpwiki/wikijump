<?php
use DB\PagePeer;

class CreateAccountModule extends SmartyModule
{
    public function build($runData)
    {
        $site = $runData->getTemp("site");
        $page = PagePeer::instance()->selectByName($site->getSiteId(), "system:create-account");
        if ($page != null) {
            $runData->contextAdd("content", $page->getCompiled()->getText());
        }
    }
}
