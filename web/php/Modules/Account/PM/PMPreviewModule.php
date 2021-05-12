<?php

namespace Wikidot\Modules\Account\PM;




use Wikidot\DB\PrivateMessage;
use Wikidot\Utils\AccountBaseModule;
use Wikidot\Utils\WikiTransformation;

use Wikijump\Services\Wikitext\ParseRenderMode;

use function Wikijump\Services\Wikitext\getWikitext;

class PMPreviewModule extends AccountBaseModule
{

    public function build($runData)
    {

        $pl = $runData->getParameterList();
        $source = $pl->getParameterValue("source");
        $subject = $pl->getParameterValue("subject");
        $toUserId = $pl->getParameterValue("to_user_id");

        $wt = getWikitext(ParseRenderMode::DIRECT_MESSAGE, null);
        $body = $wt->renderHtml($source)->html;

        $message = new PrivateMessage();
        $message->setFromUserId($runData->getUserId());
        $message->setToUserId($toUserId);
        $message->setBody($body);
        $message->setSubject($subject);

        $runData->contextAdd("message", $message);
    }
}
