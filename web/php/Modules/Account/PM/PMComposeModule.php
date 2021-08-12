<?php

declare(strict_types=1);

namespace Wikidot\Modules\Account\PM;


use Wikidot\Utils\AccountBaseModule;
use Wikidot\Utils\ProcessException;
use Wikijump\Models\User;
use Wikijump\Models\UserMessage;

/**
 * AJAX view for message composition.
 * @package Wikidot\Modules\Account\PM
 */
class PMComposeModule extends AccountBaseModule
{

    /**
     * Write or resume writing a private message.
     * @param $runData
     * @throws ProcessException
     */
    public function build($runData)
    {
        $user = $runData->user();

        $pl = $runData->getParameterList();
        $replyMessageId = $pl->getParameterValue('replyMessageId', 'AMODULE');

        $continueMessageId = $pl->getParameterValue('continueMessageId', 'AMODULE');
        $toUserId = $pl->getParameterValue('toUserId');

        if ($replyMessageId)
        {
            $message = UserMessage::find($replyMessageId);

            if ($message == null || $message->recipient->id != $user->id)
            {
                throw new ProcessException(_('Error getting orginal message.'), 'no_reply_message');
            }

            $subject = $message->subject;

            /**
             * If the subject doesn't already start with `Re: `, add it.
             * Otherwise, leave it alone. (There should only be one `Re: ` in a subject.)
             */
            if(str_starts_with('Re: ', $subject) === false)
            {
                $subject = 'Re: ' . $subject;
            }

            $runData->ajaxResponseAdd('toUserId', $message->sender->id);
            $runData->ajaxResponseAdd('toUserName', $message->sender->username);
            $runData->contextAdd('subject', $subject);
        }
        elseif ($continueMessageId)
        {
            $message = UserMessage::find($continueMessageId);

            if ($message == null || $message->sender->id != $user->id)
            {
                throw new ProcessException(_('Error getting orginal message.'), 'no_reply_message');
            }
            if ($message->recipient->id !== null)
            {
                $runData->ajaxResponseAdd('toUserId', $message->recipient->id);
                $runData->ajaxResponseAdd('toUserName', $message->recipient->username);
            }

            $runData->contextAdd('body', $message->body);
            $runData->contextAdd('subject', $message->subject);
        }
        elseif ($toUserId !== null)
        {
            $toUser = User::find($toUserId);
            $runData->ajaxResponseAdd('toUserId', $toUser->id);
            $runData->ajaxResponseAdd('toUserName', $toUser->username);
        }

        $runData->contextAdd('user', $user);
    }
}
