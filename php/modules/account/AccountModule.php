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

class AccountModule extends AccountBaseModule {
	
	protected $processPage = true;
	
	public function isAllowed($runData){
		
		return true;	
	}
	
	public function build($runData){
		if(!$runData->getUser()){
			$runData->setModuleTemplate('account/AccountNotLoggedInModule');
			return;
		}
		
		$user = $runData->getUser();
		$runData->contextAdd("user",$user);	
		
		$pl = $runData->getParameterList();
		$start = $pl->getParameterValue("start");
		if($start){
			$runData->contextAdd("start", $start);	
		}
		$composeTo = $pl->getParameterValue("composeto");
		if($composeTo){
			$runData->contextAdd("composeTo", $composeTo);
		}
		$inboxMessage = $pl->getParameterValue("inboxmessage");
		if($inboxMessage){
			$runData->contextAdd("inboxMessage", $inboxMessage);
		}
		// put the key too
		$runData->contextAdd("rsaKey", CryptUtils::modulus());
		$this->extraJs[] = '/common--javascript/crypto/rsa.js';

	}
	
	public function processPage($out, $runData){
		$out = preg_replace("/<div id=\"page-title\">(.*?)<\/div>/is",'',$out, 1);
		return $out;	
	}
	
}
