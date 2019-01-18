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

class PageRedirectModule extends SmartyModule {
	
	public function build($runData){
		$pl = $runData->getParameterList();
		
		$noRedirect = (bool) $pl->getParameterValue("noredirect");
		
		if($runData->isAjaxMode()){
			$noRedirect = true;
		}
		
		$target = trim($pl->getParameterValue("destination"));
		
		if($target == ""){
			throw new ProcessException(_('No redirection destination specified. Please use the destination="page-name" or destination="url" attribute.'));	
		}
		
		$currentUri = $_SERVER['REQUEST_URI'];
		
		if(!$noRedirect){	
			// ok, redirect!!!

			// check if mapping should be done.
			if($target{strlen($target)-1} === '/' && strpos($currentUri, '/',1)){
				$map = true;
			}else{
				$map = false;
			}
			
			// check if $target is an URI or just a page name
			if(!strpos($target, '://')){
				$target = WDStringUtils::toUnixName($target);
				$target = '/'.$target;
				if($map) {$target .= '/';}
			}
			
			if($map){
				// use more advanced mapping
				
				//strip page name and take the remaining part
				$mappedUri = substr($currentUri, strpos($currentUri, '/',1)+1);
				$target .= $mappedUri;
				
			}
			
			header('HTTP/1.1 301 Moved Permanently');
			header('Location: '.$target);
			exit();
		}else{
			$runData->contextAdd("target", $target);	
		}	
	}
	
}
