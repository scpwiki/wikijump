<?php

namespace Wikidot\Modules\History;

use Ozone\Framework\SmartyModule;
use Wikidot\DB\PageRevisionPeer;
use Wikidot\Utils\ProcessException;
use Wikijump\Services\Wikitext\ParseRenderMode;
use Wikijump\Services\Wikitext\WikitextBackend;

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

        // Why doesn't this pass in page data?
        $wt = WikitextBackend::make(ParseRenderMode::PAGE, null);
        $content = $wt->renderHtml($source)->html;

        $runData->contextAdd("pageContent", $content);
        $runData->contextAdd("revision", $revision);
        $runData->contextAdd("metadata", $metadata);
        $runData->ajaxResponseAdd("title", $metadata->getTitle());
    }
}
