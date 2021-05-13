<?php

namespace Wikidot\Actions;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Database\Database;
use Ozone\Framework\JSONService;
use Ozone\Framework\ODate;
use Ozone\Framework\SmartyAction;

use Wikidot\DB\PrivateMessage;
use Wikidot\DB\PrivateMessagePeer;
use Wikidot\Utils\NotificationMaker;
use Wikidot\Utils\ProcessException;
use Wikidot\Utils\WDPermissionException;
use Wikidot\Utils\WDPermissionManager;
use Wikijump\Models\User;
use Wikijump\Services\Wikitext\ParseRenderMode;
use Wikijump\Services\Wikitext\WikitextBackend;

class PMAction extends SmartyAction
{

    public function isAllowed($runData)
    {
        if ($runData->getUserId() === null) {
            throw new WDPermissionException(_("You should be logged in in order to send messages."));
        }
        return true;
    }

    public function perform($r)
    {
    }

    public function checkCanEvent($runData)
    {
        $pl = $runData->getParameterList();
        $toUserId = $pl->getParameterValue("userId");

        if ($toUserId === null || !is_numeric($toUserId)) {
            throw new ProcessException(_("Error selecting user."), "no_user");
        }

        $user = $runData->getUser();
        $toUser = User::find($toUserId);

        if ($toUser == null) {
            throw new ProcessException(_("Error selecting user."), "no_user");
        }

        return WDPermissionManager::instance()->hasPmPermission($user, $toUser);
    }

    public function sendEvent($runData)
    {
        $pl = $runData->getParameterList();
        $source = $pl->getParameterValue("source");
        $subject = $pl->getParameterValue("subject");

        if ($subject === '') {
            $subject = '(No subject)';
        }

        $db = Database::connection();
        $db->begin();

        $toUserId = $pl->getParameterValue("to_user_id");

        // TODO: validation. also check if user exists
        $toUser = User::find($toUserId);
        if ($toUser == null) {
            $message = __("The recipient does not exist.");
            throw new ProcessException($message, "no_recipient");
        }

        // check if allowed

        $fromUser = $runData->getUser();

        WDPermissionManager::instance()->hasPmPermission($fromUser, $toUser);

        // compile content
        $wt = WikitextBackend::make(PageRenderMode::DIRECT_MESSAGE, null);
        $body = $wt->renderHtml($source)->html;

        $message = new PrivateMessage();
        $message->setDate(new ODate());
        $message->setFromUserId($runData->getUserId());
        $message->setToUserId($toUserId);

        $message->setSubject($subject);
        $message->setBody($body);
        $message->setFlag(0); // 0 for inbox

        $message->save();

        NotificationMaker::instance()->privateMessageNotification($message);

        //also make a copy for "sent" folder

        $message->setNew(true);
        $message->setMessageId(null);
        $message->setFlag(1); //1 for sent

        $message->save();

        $db->commit();
    }

    public function saveDraftEvent($runData)
    {
        $pl = $runData->getParameterList();
        $source = $pl->getParameterValue("source");
        $subject = $pl->getParameterValue("subject");

        $toUserId = $pl->getParameterValue("to_user_id");

        // saving source only
        $body = $source;

        $db = Database::connection();
        $db->begin();

        $message = new PrivateMessage();
        $message->setDate(new ODate()); // date of saving draft
        $message->setFromUserId($runData->getUserId());
        $message->setToUserId($toUserId);

        $message->setSubject($subject);
        $message->setBody($body);
        $message->setFlag(2); // 2 for draft

        $message->save();

        $db->commit();
    }

    public function removeSelectedInboxEvent($runData)
    {
        $userId = $runData->getUserId();
        $c = new Criteria();
        $c->add("to_user_id", $userId);
        $c->add("flag", 0);

        $selected = $runData->getParameterList()->getParameterValue("selected");
        $json = new JSONService(SERVICES_JSON_LOOSE_TYPE);
        $selected = $json->decode($selected);

        $db = Database::connection();
        $db->begin();

        $c2 = new Criteria();
        foreach ($selected as $s) {
            $c2->addOr("message_id", $s);
        }
        $c->addCriteriaAnd($c2);

        PrivateMessagePeer::instance()->delete($c);

        $db->commit();
    }

    public function removeInboxMessageEvent($runData)
    {
        $messageId = $runData->getParameterList()->getParameterValue("message_id");
        $userId = $runData->getUserId();

        $db = Database::connection();
        $db->begin();

        $c = new Criteria();
        $c->add("message_id", $messageId);
        $c->add("to_user_id", $userId);
        $c->add("flag", 0);

        PrivateMessagePeer::instance()->delete($c);
        $c = new Criteria();
        $c->add("to_user_id", $userId);
        $c->add("message_id", $messageId, "<");
        $c->add("flag", 0);
        $c->addOrderDescending("message_id");

        $mid = PrivateMessagePeer::instance()->selectOne($c);
        if ($mid == null) {
                $c = new Criteria();
            $c->add("to_user_id", $userId);
            $c->add("message_id", $messageId, ">");
            $c->add("flag", 0);
            $c->addOrderAscending("message_id");

            $mid = PrivateMessagePeer::instance()->selectOne($c);
        }

        if ($mid !== null) {
            $runData->ajaxResponseAdd("messageId", $mid->getMessageId());
        }

        $db->commit();
    }

    public function removeSentMessageEvent($runData)
    {
        $messageId = $runData->getParameterList()->getParameterValue("message_id");
        $userId = $runData->getUserId();

        $db = Database::connection();
        $db->begin();

        $c = new Criteria();
        $c->add("message_id", $messageId);
        $c->add("from_user_id", $userId);
        $c->add("flag", 1);

        PrivateMessagePeer::instance()->delete($c);
        $c = new Criteria();
        $c->add("from_user_id", $userId);
        $c->add("message_id", $messageId, "<");
        $c->add("flag", 1);
        $c->addOrderDescending("message_id");

        $mid = PrivateMessagePeer::instance()->selectOne($c);
        if ($mid == null) {
                $c = new Criteria();
            $c->add("from_user_id", $userId);
            $c->add("message_id", $messageId, ">");
            $c->add("flag", 1);
            $c->addOrderAscending("message_id");

            $mid = PrivateMessagePeer::instance()->selectOne($c);
        }

        if ($mid !== null) {
            $runData->ajaxResponseAdd("messageId", $mid->getMessageId());
        }

        $db->commit();
    }

    public function removeSelectedSentEvent($runData)
    {
        $userId = $runData->getUserId();
        $c = new Criteria();
        $c->add("from_user_id", $userId);
        $c->add("flag", 1);

        $selected = $runData->getParameterList()->getParameterValue("selected");
        $json = new JSONService(SERVICES_JSON_LOOSE_TYPE);
        $selected = $json->decode($selected);

        $db = Database::connection();
        $db->begin();

        $c2 = new Criteria();
        foreach ($selected as $s) {
            $c2->addOr("message_id", $s);
        }
        $c->addCriteriaAnd($c2);

        PrivateMessagePeer::instance()->delete($c);

        $db->commit();
    }

    public function removeDraftsMessageEvent($runData)
    {
        $messageId = $runData->getParameterList()->getParameterValue("message_id");
        $userId = $runData->getUserId();

        $db = Database::connection();
        $db->begin();

        $c = new Criteria();
        $c->add("message_id", $messageId);
        $c->add("from_user_id", $userId);
        $c->add("flag", 2);

        PrivateMessagePeer::instance()->delete($c);
        $c = new Criteria();
        $c->add("from_user_id", $userId);
        $c->add("message_id", $messageId, "<");
        $c->add("flag", 2);
        $c->addOrderDescending("message_id");

        $mid = PrivateMessagePeer::instance()->selectOne($c);
        if ($mid == null) {
                $c = new Criteria();
            $c->add("from_user_id", $userId);
            $c->add("message_id", $messageId, ">");
            $c->add("flag", 2);
            $c->addOrderAscending("message_id");

            $mid = PrivateMessagePeer::instance()->selectOne($c);
        }

        if ($mid !== null) {
            $runData->ajaxResponseAdd("messageId", $mid->getMessageId());
        }

        $db->commit();
    }

    public function removeSelectedDraftsEvent($runData)
    {
        $userId = $runData->getUserId();
        $c = new Criteria();
        $c->add("from_user_id", $userId);
        $c->add("flag", 2);

        $selected = $runData->getParameterList()->getParameterValue("selected");
        $json = new JSONService(SERVICES_JSON_LOOSE_TYPE);
        $selected = $json->decode($selected);

        $db = Database::connection();
        $db->begin();

        $c2 = new Criteria();
        foreach ($selected as $s) {
            $c2->addOr("message_id", $s);
        }
        $c->addCriteriaAnd($c2);

        PrivateMessagePeer::instance()->delete($c);

        $db->commit();
    }
}
