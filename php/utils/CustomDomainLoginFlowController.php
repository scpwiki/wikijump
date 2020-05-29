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
 * @version $Id: UploadedFileFlowController.php,v 1.5 2008/08/01 14:00:27 quake Exp $
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

class CustomDomainLoginFlowController extends WikidotController {

	static public $controllerUrl = "/domainauth.php";
	
	protected function redirectConfirm($url) {
		$this->redirect(self::$controllerUrl, array("confirm" => "cookie", "url" => $url));
	}
	
	protected function cookieError($url) {
		$url = htmlspecialchars($url);
		$this->setContentTypeHeader("text/html");
		echo "<p>Can't proceed, you should accept cookies for this domain.</p>";
		echo "<p>Then you can go back to $url</p>";
	}
	
	public function process() {

		Ozone ::init();
		
		$runData = new RunData();
		$runData->init();
		Ozone::setRunData($runData);
		
		$url = $_GET["url"];
		$confirm = isset($_GET["confirm"]);
		$setie = isset($_GET["setiecookie"]);
		$siteHost = $_SERVER['HTTP_HOST'];
		
		$site = $this->siteFromHost($siteHost, true, true);
		
		if ($setie) {
			
			if ($siteHost != GlobalProperties::$URL_DOMAIN) {
				$this->siteNotExists();
			}
			
			$runData->handleSessionStart();
			if ($runData->getUser()) {
				setcookie(GlobalProperties::$SESSION_COOKIE_NAME_IE, $runData->getSessionId(), null, '/');
			} else {
				setcookie(GlobalProperties::$SESSION_COOKIE_NAME_IE, "ANONYMOUS", null, '/');
			}
			$this->redirect($url);
			
		} else {
		
			if (! $site) {
				$this->siteNotExists();
				return;
			}
		
			if (! $confirm) {
				
				$user_id = $_GET["user_id"];
				$skey =  $_GET["skey"];
				
				$session = $runData->getSessionFromDomainHash($skey, $_SERVER['HTTP_HOST'], $user_id);
				
				if ($session) {
					setcookie(GlobalProperties::$SESSION_COOKIE_NAME, "_domain_cookie_${user_id}_${skey}", null, '/', GlobalProperties::$SESSION_COOKIE_DOMAIN);
					$this->redirectConfirm($url);
				} else {
					$this->redirect($url);
				}
				
			} else {
				
				// checking if cookie exists
				
				$runData->handleSessionStart();
				
				if ($runData->getUser()) {
					$this->redirect($url);
				} else {
					$this->cookieError($url);
				}
			}
			
		}
		
	}
}
