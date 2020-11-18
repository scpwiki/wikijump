<?php
use DB\PrivateMessagePeer;

class PMSentMessageModule extends AccountBaseModule
{

    public function build($runData)
    {

        $userId = $runData->getUserId();
        $pl = $runData->getParameterList();
        $messageId = $pl->getParameterValue("message_id");

        $message = PrivateMessagePeer::instance()->selectByPrimaryKey($messageId);
        if ($message->getFromUserId() != $userId) {
            throw new ProcessException(_("Error selecting message."), "no_message");
        }

        $runData->contextAdd("message", $message);

        // get next & previous message
        $messageId = $message->getMessageId();
        $c = new Criteria();
        $c->add("from_user_id", $userId);
        $c->add("message_id", $messageId, ">");
        $c->add("flag", 1);
        $c->addOrderAscending("message_id");

        $newerMessage = PrivateMessagePeer::instance()->selectOne($c);

        $c = new Criteria();
        $c->add("from_user_id", $userId);
        $c->add("message_id", $messageId, "<");
        $c->add("flag", 1);
        $c->addOrderDescending("message_id");

        $olderMessage = PrivateMessagePeer::instance()->selectOne($c);

        $runData->contextAdd("newerMessage", $newerMessage);
        $runData->contextAdd("olderMessage", $olderMessage);
    }
}
