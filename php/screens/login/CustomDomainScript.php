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

class CustomDomainScript extends SmartyScreen {
	
	public function build($runData){
		
		// check first for standard cookie name
		$user = $runData->getUser();
		$anon = false;
		
		if (! $user) {
			// check the ie cookie then
			GlobalProperties::$SESSION_COOKIE_NAME = GlobalProperties::$SESSION_COOKIE_NAME_IE;
			$runData->handleSessionStart();
			$user = $runData->getUser();
			$anon = ($_COOKIE[GlobalProperties::$SESSION_COOKIE_NAME_IE] == "ANONYMOUS");
		}
		
		if ($user) {
		
			$site_id = (int) $runData->getParameterList()->getParameterValue("site_id");
			$site = DB_SitePeer::instance()->selectByPrimaryKey($site_id);
			
			if ($site && $site->getCustomDomain()) {
				$skey = $runData->generateSessionDomainHash($site->getCustomDomain());
				$proto = ($_SERVER["HTTPS"]) ? "https" : "http";
				$domain = $site->getCustomDomain();
				$runData->contextAdd("redir", "$proto://$domain" . CustomDomainLoginFlowController::$controllerUrl . "?" . http_build_query(array("user_id" => $user->getUserId(), "skey" => $skey)));
			}
			
		} elseif (! $anon) {
			
			// no session found -- try to redirect to set ie cookie
			$proto = ($_SERVER["HTTPS"]) ? "https" : "http";
			$runData->contextAdd("redirIE", $proto . '://' . GlobalProperties::$URL_HOST . CustomDomainLoginFlowController::$controllerUrl . '?' . http_build_query(array("url" => $url, "setiecookie" => true)));
			
		}
		
	}
	
}
