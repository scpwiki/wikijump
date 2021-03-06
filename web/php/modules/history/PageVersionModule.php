<?php
use DB\PageRevisionPeer;

class PageVersionModule extends SmartyModule
{

    public function build($runData)
    {
        $revisionId = $runData->getParameterList()->getParameterValue("revision_id");

        $revision = PageRevisionPeer::instance()->selectByPrimaryKey($revisionId);
        if ($revision == null) {
            throw new ProcessException(_("Revision error"), "revision_error");
        }

        $metadata = $revision->getMetadata();

        $source = $revision->getSourceText();

        $tr = new WikiTransformation();
        $content = $tr->processSource($source);

        $runData->contextAdd("pageContent", $content);
        $runData->contextAdd("revision", $revision);
        $runData->contextAdd("metadata", $metadata);
        $runData->ajaxResponseAdd("title", $metadata->getTitle());
    }
}
