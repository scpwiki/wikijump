<?php
declare(strict_types=1);

namespace Wikidot\Modules\Account\PM;

use Wikidot\Utils\AccountBaseModule;
use Wikijump\Models\UserMessage;
use Wikijump\Services\Deepwell\DeepwellService;
use Wikijump\Services\Wikitext\ParseRenderMode;

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
        $source = $runData->get('source');
        $body = DeepwellService::getInstance()->renderHtml(ParseRenderMode::DIRECT_MESSAGE, $source, null);

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
