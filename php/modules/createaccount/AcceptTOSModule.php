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

class AcceptTOSModule extends SmartyModule {
	
	public function isAllowed($runData){
		if($runData->getUserId() !== null){
			throw new ProcessException(_("You are already logged in. Why would you want to create a new account?"), "logged_in"); 	
		}	
		return true;
	}
	
	public function build($runData){
		$runData->sessionAdd("rstep", -1);
		// get terms of service.

		// also set the crypto things
		
		$runData->ajaxResponseAdd("key", CryptUtils::modulus());
		
		// get the TOS content
		
		$pageName = "legal:terms-of-service";
		$siteName = "www";
		
		$c = new Criteria();
		$c->add("unix_name", $siteName);
		$site = DB_SitePeer::instance()->selectOne($c);
		
		$page = DB_PagePeer::instance()->selectByName($site->getSiteId(), $pageName);
		// get content
		$content = $page->getCompiled()->getText();
		
		// remove toc ;-)
		$content = preg_replace(';<table style=".*?id="toc".*?</table>;s', '', $content, 1);
		$content = preg_replace(';<a ([^>]*)>;s', '<a \\1 target="_blank">', $content);
		
		$runData->contextAdd("tosContent", $content);
		
	}
	
}
