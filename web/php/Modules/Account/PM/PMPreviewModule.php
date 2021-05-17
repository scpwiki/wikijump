<?php

namespace Wikidot\Modules\Account\PM;

use Wikidot\DB\PrivateMessage;
use Wikidot\Utils\AccountBaseModule;
use Wikijump\Services\Wikitext\ParseRenderMode;
use Wikijump\Services\Wikitext\WikitextBackend;

class PMPreviewModule extends AccountBaseModule
{

    public function build($runData)
    {
        $pl = $runData->getParameterList();
        $source = $pl->getParameterValue("source");
        $subject = $pl->getParameterValue("subject");
        $toUserId = $pl->getParameterValue("to_user_id");

        $wt = WikitextBackend::make(ParseRenderMode::DIRECT_MESSAGE, null);
        $body = $wt->renderHtml($source)->body;

        $message = new PrivateMessage();
        $message->setFromUserId($runData->getUserId());
        $message->setToUserId($toUserId);
        $message->setBody($body);
        $message->setSubject($subject);

        $runData->contextAdd("message", $message);
    }
}
