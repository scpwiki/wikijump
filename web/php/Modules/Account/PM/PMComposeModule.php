<?php

namespace Wikidot\Modules\Account\PM;




use Wikidot\DB\PrivateMessagePeer;
use Wikidot\DB\OzoneUserPeer;
use Wikidot\Utils\AccountBaseModule;
use Wikidot\Utils\ProcessException;

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

            if ($message == null || $message->getToUserId() != $user->getUserId()) {
                throw new ProcessException(_("Error getting orginal message."), "no_reply_message");
            }
            $runData->ajaxResponseAdd("toUserId", $message->getFromUserId());
            $runData->ajaxResponseAdd("toUserName", $message->getFromUser()->getNickName());
            $subject = $message->getSubject();
            $subject = preg_replace("/^Re: /", '', $subject);
            $runData->contextAdd("subject", "Re: ".$subject);
        } elseif ($continueMessageId) {
            $message = PrivateMessagePeer::instance()->selectByPrimaryKey($continueMessageId);

            if ($message == null || $message->getFromUserId() != $user->getUserId()) {
                throw new ProcessException(_("Error getting orginal message."), "no_reply_message");
            }
            if ($message->getToUserId() !== null) {
                $runData->ajaxResponseAdd("toUserId", $message->getToUserId());
                $runData->ajaxResponseAdd("toUserName", $message->getToUser()->getNickName());
            }
            $runData->contextAdd("body", $message->getBody());
            $runData->contextAdd("subject", $message->getSubject());
        } elseif ($toUserId !== null) {
            $toUser = OzoneUserPeer::instance()->selectByPrimaryKey($toUserId);
            $runData->ajaxResponseAdd("toUserId", $toUser->getUserId());
            $runData->ajaxResponseAdd("toUserName", $toUser->getNickName());
        }

        $user = $runData->getUser();

        $runData->contextAdd("user", $user);
    }
}
