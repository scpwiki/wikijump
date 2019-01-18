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
 * @category Ozone
 * @package Ozone_Web
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

/**
 * Abstract class for smarty-based screens.
 */
abstract class SmartyScreen extends Screen{
	
	protected $screenCacheSettings;
	
	public function __construct(){}
	
	 /**
	  * This method renders the page.
	  * 
	  */
	 public function render($runData){

	 	## render!
	 	
	 	if($runData->getScreenTemplate()==null || $runData->getPage()->getLayout()==null){
	 		return null;	
	 	}
	 	
	 	$smarty = Ozone::getSmarty();
	 	
	 	$templateFile  = PathManager::screenTemplate($runData->getScreenTemplate());

	 	//////////

		$updateLayoutContentLater = false;
		$mainContent=null;
	 	if(!file_exists($templateFile)){
	 		// some error please!
	 		$runData->setScreenTemplate("DefaultError");
	 		$runData->addErrorMessage("Taka strona nie istnieje.");
	 		$templateFile  = PathManager::screenTemplate($runData->getScreenTemplate());
	 		
	 	} else {
	 		
	 		// process the cache!!!
	 		$cacheSettings = $this->getScreenCacheSettings();
			if($runData->getRequestMethod() == "GET" 
					&& $runData->getAction() == null 
					&& $cacheSettings != null 
					&& $cacheSettings->isScreenCacheable($runData)){
	 			
	 			$content = ScreenCacheManager::instance()->cachedScreen($runData, $this->getScreenCacheSettings());
				if($content != null && $content != ""){
					$mainContent = $content;	
				} else {
					$updateScreenContentLater = true;	
					// 	run user's method "build"
	 				$this->build($runData);
				}
				// cache end!!! (for now...)
			} else {
				// 	run user's method "build"
	 			$this->build($runData);	
			}
	 	}
	 	
	 	// repeat in case sceen template has changed...
	 	$templateFile  = PathManager::screenTemplate($runData->getScreenTemplate());
	 	 	
	 	// put context into context
	 	
	 	$context = $runData->getContext();
	 	if($context !== null){
	 		foreach($context as $key => $value){
		 		$smarty->assign($key, $value);
	 		}
	 	}
	 	
	 	$page = $runData->getPage();
	 	$smarty->assign("page", $page);

	 	// put errorMessages and messages into the smarty's context as well.
	 	$dataMessages = $runData->getMessages();	
	 	$dataErrorMessages = $runData->getErrorMessages();
	 	if(count($dataMessages) > 0) {
	 		$smarty->assign('data_messages', $dataMessages);	
	 	}

	 	if(count($dataErrorMessages) > 0) {
	 		$smarty->assign('data_errorMessages', $dataErrorMessages);	
	 	}

	 	if($mainContent == null){	 	
	 		$mainContent = $smarty->fetch($templateFile);
	 	}
	 	
	 	if($updateScreenContentLater){
	 		// update the cached content in the database	
	 		ScreenCacheManager::instance()->updateCachedScreen($runData, $mainContent);
	 	}

	 	$layoutFile = PathManager::layoutTemplate($page->getLayout());
	 	$smarty->assign("screen_placeholder", $mainContent);
		$page->setStyleSelector(1);
	 	$out = $smarty->fetch($layoutFile);

	 	return $out;
	 	
	 }
	 
	 public function getSelf(){
	 	return $this;
	 }
	 	
	 public function getScreenCacheSettings(){
	 	return $this->screenCacheSettings;	
	 }
	 
	abstract public function build($runData);
}
