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

class UserInfoModule extends SmartyLocalizedModule {
	
	private $user; // nasty hack again...
	
	protected $processPage = true;
	
	public function build($runData){

		// a hack to get unix user name 
		$qs =  $_SERVER['QUERY_STRING'];
		$splited = explode("/",$qs);
		
		// WARNING!!! this is a hack! not a proper use of ParameterList object!
		$userUnixName = $splited[3];
		
		if($userUnixName == null || $userUnixName == ''){
			throw new ProcessException(_("No user specified."), "no_user");	
		}
		
		// get user
		$c = new Criteria();
		$c->add("unix_name", $userUnixName);
		$user = DB_OzoneUserPeer::instance()->selectOne($c);
		
		if($user == null){
			throw new ProcessException(_("User does not exist."));
		}
		
		$runData->contextAdd("user", $user);
		$runData->contextAdd("userUnixName", $userUnixName); 
		$runData->contextAdd("userId", $user->getUserId());
		
		$this->user = $user;

		// get the referring page too in case one wants to 
		// flag an abusive user. than we set site_id of the flag
		// to the site which the user comes from if
		// this is a wikidot site.
		
		$referer = $_SERVER['HTTP_REFERER'];
		
		if($referer){
			$referer = parse_url($referer);
			$referer = $referer['host'];	
		}
		
		$runData->contextAdd("referer", $referer);
		
		$runData->contextAdd("uu", $runData->getUser());
	}
	
	public function processPage($out, $runData){
		// modify title of the page
		$user = $this->user;
		if($user != null){
			$out = preg_replace("/<title>(.+?)<\/title>/is","<title>".GlobalProperties::$SERVICE_NAME.": ".$user->getNickName()."</title>",$out);
			$out = preg_replace("/<div id=\"page-title\">(.*?)<\/div>/is",'',$out, 1);
		}
		return $out;	
	}
	
}
