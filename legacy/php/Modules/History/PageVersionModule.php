<?php

namespace Wikidot\Modules\History;

use Ozone\Framework\SmartyModule;
use Wikidot\DB\PageRevisionPeer;
use Wikidot\Utils\ProcessException;
use Wikijump\Services\Deepwell\DeepwellService;
use Wikijump\Services\Wikitext\ParseRenderMode;

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
        $content = DeepwellService::getInstance()->renderHtml(ParseRenderMode::PAGE, $source, null);

        $runData->contextAdd("pageContent", $content);
        $runData->contextAdd("revision", $revision);
        $runData->contextAdd("metadata", $metadata);
        $runData->ajaxResponseAdd("title", $metadata->getTitle());
    }
}
