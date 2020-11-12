<?php
use DB\FilePeer;

class ImageAttachedFileModule extends SmartyModule
{

    public function build($runData)
    {
        $pl = $runData->getParameterList();

        $pageId = $pl->getParameterValue("pageId");

        $c = new Criteria();
            $c->add("page_id", $pageId);
            $c->add("has_resized", true);
            $c->addOrderAscending("filename");
            $files = FilePeer::instance()->select($c);

            $runData->contextAdd("files", $files);
    }
}
