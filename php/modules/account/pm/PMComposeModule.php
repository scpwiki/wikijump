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

class PMComposeModule extends AccountBaseModule {
	
	public function build($runData){
		
		$user = $runData->getUser();
		
		$pl = $runData->getParameterList();
		$replyMessageId = $pl->getParameterValue("replyMessageId", "AMODULE");
		
		$continueMessageId = $pl->getParameterValue("continueMessageId", "AMODULE");
		$toUserId = $pl->getParameterValue("toUserId");

		if($replyMessageId){
			$message = DB_PrivateMessagePeer::instance()->selectByPrimaryKey($replyMessageId);	
			
			if($message == null || $message->getToUserId() != $user->getUserId()){
				throw new ProcessException(_("Error getting orginal message."), "no_reply_message");	
			}
			$runData->ajaxResponseAdd("toUserId", $message->getFromUserId());
			$runData->ajaxResponseAdd("toUserName", $message->getFromUser()->getNickName());
			$subject = $message->getSubject();
			$subject = preg_replace("/^Re: /", '', $subject);
			$runData->contextAdd("subject", "Re: ".$subject);
		}elseif($continueMessageId){
			$message = DB_PrivateMessagePeer::instance()->selectByPrimaryKey($continueMessageId);	
			
			if($message == null || $message->getFromUserId() != $user->getUserId()){
				throw new ProcessException(_("Error getting orginal message."), "no_reply_message");	
			}
			if($message->getToUserId() !== null){
				$runData->ajaxResponseAdd("toUserId", $message->getToUserId());
				$runData->ajaxResponseAdd("toUserName", $message->getToUser()->getNickName());
			}
			$runData->contextAdd("body", $message->getBody());
			$runData->contextAdd("subject", $message->getSubject());
			
		}elseif($toUserId !== null){

			$toUser = DB_OzoneUserPeer::instance()->selectByPrimaryKey($toUserId);
			$runData->ajaxResponseAdd("toUserId", $toUser->getUserId());
			$runData->ajaxResponseAdd("toUserName", $toUser->getNickName());	
		}
		
		$user = $runData->getUser();

		$runData->contextAdd("user", $user);
	}	
	
}
