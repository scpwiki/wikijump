<?php

declare(strict_types=1);

namespace Wikidot\Modules\Account\PM;

use Wikidot\Utils\AccountBaseModule;
use Wikidot\Utils\ProcessException;
use Wikijump\Models\UserMessage;
use Wikijump\Services\Deepwell\DeepwellService;
use Wikijump\Services\Wikitext\ParseRenderMode;

/**
 * AJAX Module for PM Drafts.
 * @package Wikidot\Modules\Account\PM
 */
class PMDraftsMessageModule extends AccountBaseModule
{

    /**
     * Retrieve the previous, current, and next draft for this user.
     * @param $runData
     * @throws ProcessException
     */
    public function build($runData)
    {
        $message = UserMessage::find($runData->get('message_id'));
        if ($message->sender->id != $runData->id()) {
            throw new ProcessException(_('Error selecting message.'), 'no_message');
        }

        $message->body = DeepwellService::getInstance()->renderHtml(ParseRenderMode::DIRECT_MESSAGE, $message->body, null);

        $runData->contextAdd('message', $message);

        $newerMessage = UserMessage::drafts($runData->user())
            ->where('id', '>', $message->created_at)
            ->first();

        $olderMessage = UserMessage::drafts($runData->user())
            ->where('id', '<', $message->created_at)
            ->first();

        $runData->contextAdd('newerMessage', $newerMessage);
        $runData->contextAdd('olderMessage', $olderMessage);
    }
}
