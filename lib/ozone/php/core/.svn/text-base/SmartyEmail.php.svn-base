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
 * @package Ozone_Email
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */
 
/**
 * Email class that uses Smarty and PHPMailer.
 *
 */
class SmartyEmail extends PHPMailerWrap{

	private $bodyTemplate;
	private $renderedBody;
	private $context;
	
	public function setBodyTemplate($templateName){
		$this->bodyTemplate = $templateName;	
	}	
	
	public function getBodyTemplate(){
		return $this->bodyTemplate;	
	}
	
	public function contextDel($key=null) {
		if($key != null){
			unset($this->context["$key"]);
		} else {
			$this->context = array ();
		}
	}
	
	public function contextAdd($key, $value){
		$this->context["$key"] = $value;
	}
	
	public function contextGet($key){
		return $this->context["$key"];	
	}
	
	public function getContext(){
		return $this->context;	
	}
		
	public function send(){
		// get the template file
		$templateFile = PathManager::emailTemplate($this->bodyTemplate);
		
		// get 	the Smarty engine
		$smarty = new OzoneSmarty();
		
		$context = $this->context;
	 	if($context !== null){
	 		foreach($context as $key => $value){
		 		$smarty->assign($key, $value);
	 		}
	 	}
	 	
	 	$body = $smarty->fetch($templateFile);
	 	
	 	$this->setBody($body);
	 	
	 	if (parent::send()) {
			return true;
		} else {
			return false;
		}
		
	}

}
