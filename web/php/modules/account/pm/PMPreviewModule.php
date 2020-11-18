<?php
use DB\PrivateMessage;

class PMPreviewModule extends AccountBaseModule
{

    public function build($runData)
    {

        $pl = $runData->getParameterList();
        $source = $pl->getParameterValue("source");
        $subject = $pl->getParameterValue("subject");
        $toUserId = $pl->getParameterValue("to_user_id");

        $wt = new WikiTransformation();
        $wt->setMode('pm');
        $result = $wt->processSource($source);

        $body = $result;

        $message = new PrivateMessage();
        $message->setFromUserId($runData->getUserId());
        $message->setToUserId($toUserId);
        $message->setBody($body);
        $message->setSubject($subject);

        $runData->contextAdd("message", $message);
    }
}
