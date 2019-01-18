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
 * Abstract class for smarty-based modules.
 *
 */
abstract class SmartyModule extends Module{

	private $template;

	public function render($runData){
	 	
	 	if($runData->getModuleTemplate() == null){return;}
	 	
	 	$this->build($runData);
	 	
	 	$template = $runData->getModuleTemplate();
	 	$templateFile  = PathManager::moduleTemplate($template);
	 	// render!
	 	
	 	$smarty = Ozone::getSmartyPlain();

	 	$page = $runData->getPage();
	 	$smarty->assign("page", $page);
	 	
	 	// put context into context
	 	
	 	$context = $runData->getContext();
	 	if($context !== null){
	 		foreach($context as $key => $value){
		 		$smarty->assign($key, $value);
	 		}
	 	}
	 	
	 	// put errorMessages and messages into the smarty's context as well.
	 	$dataMessages = $runData->getMessages();	
	 	$dataErrorMessages = $runData->getErrorMessages();
	 	if(count($dataMessages) > 0) {
	 		$smarty->assign('data_messages', $dataMessages);	
	 	}

	 	if(count($dataErrorMessages) > 0) {
	 		$smarty->assign('data_errorMessages', $dataErrorMessages);	
	 	}
	 		 	
	 	$out = $smarty->fetch($templateFile);
	 	
	 	return $out;

	 }
	 
	 public function setTemplate($template){
	 	$this->template = $template;
	 }	
	 
	 public function getTemplate(){
	 	return $this->template;	
	 }

	/**
	 * builds context 
	 */
	abstract public function build($runData);

}
