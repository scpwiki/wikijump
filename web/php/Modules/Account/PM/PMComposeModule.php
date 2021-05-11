<?php

namespace Wikidot\Modules\Account\PM;




use Wikidot\DB\PrivateMessagePeer;

use Wikidot\Utils\AccountBaseModule;
use Wikidot\Utils\ProcessException;
use Wikijump\Models\User;

class PMComposeModule extends AccountBaseModule
{

    public function build($runData)
    {

        $user = $runData->getUser();

        $pl = $runData->getParameterList();
        $replyMessageId = $pl->getParameterValue("replyMessageId", "AMODULE");

        $continueMessageId = $pl->getParameterValue("continueMessageId", "AMODULE");
        $toUserId = $pl->getParameterValue("toUserId");

        if ($replyMessageId) {
            $message = PrivateMessagePeer::instance()->selectByPrimaryKey($replyMessageId);

            if ($message == null || $message->getToUserId() != $user->id) {
                throw new ProcessException(_("Error getting orginal message."), "no_reply_message");
            }
            $runData->ajaxResponseAdd("toUserId", $message->getFromUserId());
            $runData->ajaxResponseAdd("toUserName", $message->getFromUser()->username);
            $subject = $message->getSubject();
            $subject = preg_replace("/^Re: /", '', $subject);
            $runData->contextAdd("subject", "Re: ".$subject);
        } elseif ($continueMessageId) {
            $message = PrivateMessagePeer::instance()->selectByPrimaryKey($continueMessageId);

            if ($message == null || $message->getFromUserId() != $user->id()) {
                throw new ProcessException(_("Error getting orginal message."), "no_reply_message");
            }
            if ($message->getToUserId() !== null) {
                $runData->ajaxResponseAdd("toUserId", $message->getToUserId());
                $runData->ajaxResponseAdd("toUserName", $message->getToUser()->username);
            }
            $runData->contextAdd("body", $message->getBody());
            $runData->contextAdd("subject", $message->getSubject());
        } elseif ($toUserId !== null) {
            $toUser = User::find($toUserId);
            $runData->ajaxResponseAdd("toUserId", $toUser->id);
            $runData->ajaxResponseAdd("toUserName", $toUser->username);
        }

        $user = $runData->getUser();

        $runData->contextAdd("user", $user);
    }
}
