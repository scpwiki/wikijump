<?php
/**
 * Wikidot - free wiki collaboration software
 * Copyright (c) 2008, Wikidot Inc.
 * 
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * For more information about licensing visit:
 * http://www.wikidot.org/license
 * 
 * @category Wikidot
 * @package Wikidot
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

class PMAction extends SmartyAction {
	
	public function isAllowed($runData){
		if($runData->getUserId() === null){
			throw new WDPermissionException(_("You should be logged in in order to send messages."));	
		}
		return true;	
	}
	
	public function perform($r){}
	
	public function checkCanEvent($runData){
		$pl = $runData->getParameterList();	
		$toUserId = $pl->getParameterValue("userId");
		
		if($toUserId === null || !is_numeric($toUserId)){
			throw new ProcessException(_("Error selecting user."), "no_user");	
		}
		
		$user = $runData->getUser();
		$toUser = DB_OzoneUserPeer::instance()->selectByPrimaryKey($toUserId);
		
		if($toUser == null){
			throw new ProcessException(_("Error selecting user."), "no_user");	
		}
		
		return WDPermissionManager::instance()->hasPmPermission($user, $toUser);
		
	}
	
	public function sendEvent($runData){
		$pl = $runData->getParameterList();
		$source = $pl->getParameterValue("source");
		$subject = $pl->getParameterValue("subject");
		
		if($subject == null || $subject === ''){
			$subject = "(No subject)";	
		}
		
		$db = Database::connection();
		$db->begin();

		$toUserId = $pl->getParameterValue("to_user_id");
		
		// TODO: validation. also check if user exists
		$toUser = DB_OzoneUserPeer::instance()->selectByPrimaryKey($toUserId);
		if($toUser == null){
			$message = _("The recipient does not exist.");
			throw new ProcessException($message, "no_recipient");	
		}
		
		// check if allowed

		$fromUser = $runData->getUser();
		
		WDPermissionManager::instance()->hasPmPermission($fromUser, $toUser);
		
		// compile content
		$wt = new WikiTransformation();
		$wt->setMode('pm');
		$body = $wt->processSource($source);

		$message = new DB_PrivateMessage();
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
	
	public function saveDraftEvent($runData){
		$pl = $runData->getParameterList();
		$source = $pl->getParameterValue("source");
		$subject = $pl->getParameterValue("subject");
		
		$toUserId = $pl->getParameterValue("to_user_id");

		// saving source only
		$body = $source;
		
		$db = Database::connection();
		$db->begin();
		
		$message = new DB_PrivateMessage();
		$message->setDate(new ODate()); // date of saving draft
		$message->setFromUserId($runData->getUserId());
		$message->setToUserId($toUserId);
		
		$message->setSubject($subject);
		$message->setBody($body);
		$message->setFlag(2); // 2 for draft
		
		$message->save();	
		
		$db->commit();
	}
	
	public function removeSelectedInboxEvent($runData){
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
		foreach($selected as $s){
			$c2->addOr("message_id", $s);	
		}
		$c->addCriteriaAnd($c2);
		
		DB_PrivateMessagePeer::instance()->delete($c);
			
		$db->commit();
	}
	
	public function removeInboxMessageEvent($runData){
		$messageId = $runData->getParameterList()->getParameterValue("message_id");
		$userId = $runData->getUserId();
		
		$db = Database::connection();
		$db->begin();
		
		$c = new Criteria();
		$c->add("message_id", $messageId);
		$c->add("to_user_id", $userId);
		$c->add("flag", 0);
		
		DB_PrivateMessagePeer::instance()->delete($c);
		$c = new Criteria();
		$c->add("to_user_id", $userId);
		$c->add("message_id", $messageId, "<");
		$c->add("flag", 0);
		$c->addOrderDescending("message_id");
		
		$mid = DB_PrivateMessagePeer::instance()->selectOne($c);
		if($mid == null){
				$c = new Criteria();
			$c->add("to_user_id", $userId);
			$c->add("message_id", $messageId, ">");
			$c->add("flag", 0);
			$c->addOrderAscending("message_id");
			
			$mid = DB_PrivateMessagePeer::instance()->selectOne($c);	
		}	
		
		if($mid !== null){
			$runData->ajaxResponseAdd("messageId", $mid->getMessageId());
		}
		
		$db->commit();
	}
	
	public function removeSentMessageEvent($runData){
		$messageId = $runData->getParameterList()->getParameterValue("message_id");
		$userId = $runData->getUserId();
		
		$db = Database::connection();
		$db->begin();
		
		$c = new Criteria();
		$c->add("message_id", $messageId);
		$c->add("from_user_id", $userId);
		$c->add("flag", 1);
		
		DB_PrivateMessagePeer::instance()->delete($c);
		$c = new Criteria();
		$c->add("from_user_id", $userId);
		$c->add("message_id", $messageId, "<");
		$c->add("flag", 1);
		$c->addOrderDescending("message_id");
		
		$mid = DB_PrivateMessagePeer::instance()->selectOne($c);
		if($mid == null){
				$c = new Criteria();
			$c->add("from_user_id", $userId);
			$c->add("message_id", $messageId, ">");
			$c->add("flag", 1);
			$c->addOrderAscending("message_id");
			
			$mid = DB_PrivateMessagePeer::instance()->selectOne($c);	
		}	
		
		if($mid !== null){
			$runData->ajaxResponseAdd("messageId", $mid->getMessageId());
		}
		
		$db->commit();
	}
	
	public function removeSelectedSentEvent($runData){
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
		foreach($selected as $s){
			$c2->addOr("message_id", $s);	
		}
		$c->addCriteriaAnd($c2);
		
		DB_PrivateMessagePeer::instance()->delete($c);
			
		$db->commit();
	}
	
	public function removeDraftsMessageEvent($runData){
		$messageId = $runData->getParameterList()->getParameterValue("message_id");
		$userId = $runData->getUserId();
		
		$db = Database::connection();
		$db->begin();
		
		$c = new Criteria();
		$c->add("message_id", $messageId);
		$c->add("from_user_id", $userId);
		$c->add("flag", 2);
		
		DB_PrivateMessagePeer::instance()->delete($c);
		$c = new Criteria();
		$c->add("from_user_id", $userId);
		$c->add("message_id", $messageId, "<");
		$c->add("flag", 2);
		$c->addOrderDescending("message_id");
		
		$mid = DB_PrivateMessagePeer::instance()->selectOne($c);
		if($mid == null){
				$c = new Criteria();
			$c->add("from_user_id", $userId);
			$c->add("message_id", $messageId, ">");
			$c->add("flag", 2);
			$c->addOrderAscending("message_id");
			
			$mid = DB_PrivateMessagePeer::instance()->selectOne($c);	
		}	
		
		if($mid !== null){
			$runData->ajaxResponseAdd("messageId", $mid->getMessageId());
		}
		
		$db->commit();
	}
	
	public function removeSelectedDraftsEvent($runData){
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
		foreach($selected as $s){
			$c2->addOr("message_id", $s);	
		}
		$c->addCriteriaAnd($c2);
		
		DB_PrivateMessagePeer::instance()->delete($c);
			
		$db->commit();
	}
	
}
