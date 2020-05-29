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

class PMInboxModule extends AccountBaseModule {
	
	public function build($runData){
		
		$userId = $runData->getUserId();
		
		$pl = $runData->getParameterList();
		
		$pageNo = $pl->getParameterValue("page"); 
		if($pageNo == null || $pageNo<0){$pageNo = 1;}
		
		// get inbox messages for the user
		$c = new Criteria();
		$c->add("to_user_id", $userId);
		$c->add("flag", 0); // for inbox
		
		// also count them all!
		$co = DB_PrivateMessagePeer::instance()->selectCount($c);
		
		$c->addOrderDescending("message_id");
		
		$perPage = 30;
		// limits...
		$totalPages = ceil($co/$perPage);
		if($pageNo>$totalPages){$pageNo = $totalPages;}
		$offset = ($pageNo-1) * $perPage;
		
		$c->setLimit($perPage, $offset); 
		$runData->contextAdd("totalPages", $totalPages);
		$runData->contextAdd("currentPage", $pageNo);
		
		$messages = DB_PrivateMessagePeer::instance()->select($c);
		
		$runData->contextAdd("count", $co);
		$runData->contextAdd("totalPages", $totalPages);
		$runData->contextAdd("messages", $messages);

	}
	
}
