<?php

declare(strict_types=1);

namespace Wikidot\Modules\Account\PM;

use Wikidot\Utils\AccountBaseModule;
use Wikidot\Utils\ProcessException;
use Wikijump\Models\UserMessage;

/**
 * AJAX Class for single sent message view.
 * @package Wikidot\Modules\Account\PM
 */
class PMSentMessageModule extends AccountBaseModule
{

    /**
     * Retrieve the previous, current, and next sent message.
     * @param $runData
     * @throws ProcessException
     */
    public function build($runData)
    {
        $message = UserMessage::find($runData->get('message_id'));
        if ($message->sender->id != $runData->id()) {
            throw new ProcessException(_('Error selecting message.'), 'no_message');
        }

        $runData->contextAdd('message', $message);

        $newerMessage = UserMessage::sent($runData->user())
            ->where('id', '>', $message->created_at)
            ->first();

        $olderMessage = UserMessage::sent($runData->user())
            ->where('id', '<', $message->created_at)
            ->first();

        $runData->contextAdd('newerMessage', $newerMessage);
        $runData->contextAdd('olderMessage', $olderMessage);
    }
}
