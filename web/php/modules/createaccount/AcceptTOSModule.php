<?php
use DB\SitePeer;
use DB\PagePeer;

class AcceptTOSModule extends SmartyModule
{

    public function isAllowed($runData)
    {
        if ($runData->getUserId() !== null) {
            throw new ProcessException(_("You are already logged in. Why would you want to create a new account?"), "logged_in");
        }
        return true;
    }

    public function build($runData)
    {
        $runData->sessionAdd("rstep", -1);
        // get terms of service.

        // also set the crypto things

        $runData->ajaxResponseAdd("key", CryptUtils::modulus());

        // get the TOS content

        $pageName = "legal:terms-of-service";
        $siteName = "www";

        $c = new Criteria();
        $c->add("unix_name", $siteName);
        $site = SitePeer::instance()->selectOne($c);

        $page = PagePeer::instance()->selectByName($site->getSiteId(), $pageName);
        // get content
        $content = $page->getCompiled()->getText();

        $content = preg_replace('/<table style=".*?id="toc".*?<\/table>/s', '', $content, 1);
        $content = preg_replace('/<a ([^>]*)>/s', '<a \\1 target="_blank">', $content);

        $runData->contextAdd("tosContent", $content);
    }
}
