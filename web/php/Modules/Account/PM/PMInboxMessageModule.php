<?php

declare(strict_types=1);

namespace Wikidot\Modules\Account\PM;

use Wikidot\Utils\AccountBaseModule;
use Wikidot\Utils\ProcessException;
use Wikijump\Models\UserMessage;

/**
 * AJAX Class for Single-message Inbox view
 * @package Wikidot\Modules\Account\PM
 */
class PMInboxMessageModule extends AccountBaseModule
{

    /**
     * Retrieve the previous, current, and next inbox message.
     * @param $runData
     * @throws ProcessException
     */
    public function build($runData)
    {
        /** @var UserMessage $message */
        $message = UserMessage::find($runData->get('message_id'));

        if ($message == null || $message->recipient->id != $runData->id()) {
            throw new ProcessException(_('Error selecting message.'), 'no_message');
        }

        if ($message->isUnread()) {
            $message->markAsRead();
            $message->save();
        }
        $runData->contextAdd('message', $message);

        $newerMessage = UserMessage::inbox($runData->user())
            ->where('id', '>', $message->created_at)
            ->first();

        $olderMessage = UserMessage::inbox($runData->user())
            ->where('id', '<', $message->created_at)
            ->first();

        $runData->contextAdd('newerMessage', $newerMessage);
        $runData->contextAdd('olderMessage', $olderMessage);
    }
}
