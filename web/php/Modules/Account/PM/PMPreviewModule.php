<?php
declare(strict_types=1);

namespace Wikidot\Modules\Account\PM;

use Wikidot\Utils\AccountBaseModule;
use Wikijump\Models\UserMessage;
use Wikijump\Services\Wikitext\ParseRenderMode;
use Wikijump\Services\Wikitext\WikitextBackend;

/**
 * AJAX module for previewing a PM
 * @package Wikidot\Modules\Account\PM
 */
class PMPreviewModule extends AccountBaseModule
{

    /**
     * @param $runData
     */
    public function build($runData)
    {
        $wt = WikitextBackend::make(ParseRenderMode::DIRECT_MESSAGE, null);
        $body = $wt->renderHtml($runData->get('source'))->body;

        $message = new UserMessage(
            [
                'from_user_id' => $runData->id(),
                'to_user_id' => $runData->get('to_user_id'),
                'subject' => $runData->get('subject'),
                'body' => $body
            ]
        );

        $runData->contextAdd('message', $message);
    }
}
