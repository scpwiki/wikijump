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

class MailFormModule extends CacheableModule{ //SmartyModule {
	
	public $timeOut = 600;
	
	public function build($runData){
	
		$pl = $runData->getParameterList();
		$email = $pl->getParameterValue("email", 'MODULE', 'AMODULE');
		$format = $pl->getParameterValue("module_body", 'MODULE', 'AMODULE');
		$title = $pl->getParameterValue("title", 'MODULE', 'AMODULE');
		$successPage = $pl->getParameterValue("successPage", 'MODULE', 'AMODULE');
		$outputFormat =  $pl->getParameterValue("format", 'MODULE', 'AMODULE');
		// check if email is ok
		
		if(strlen($email)<5 || strlen($email)>50 || preg_match("/^[_a-zA-Z0-9-]+(\.[_a-zA-Z0-9-]+)*@[a-zA-Z0-9-]+(\.[a-zA-Z0-9-]+)+$/", $email) ==0){
			throw new ProcessException(_('Please provide a valid email address via the email="address@example.com" parameter.'));	
		}
		
		// check if title is valid
		if($title && strlen8($title)>200){
			throw new ProcessException(_('The title="..." attribute is not valid.'));
		}
		
		// parse the format
		$fields = MailFormUtils::parseFormat($format);
		
		//stringify fields too...
		
		$runData->contextAdd("fields", $fields);
		$runData->contextAdd("rand", rand(1,100000));
		
		// save fields!
		$parmArray = $pl->asArray();
		$key = md5(serialize($parmArray)).'_'.time();
	
		DatabaseStorage::instance()->set($key, array('email' => $email, 'fields' =>$fields, 'title'=>$title, 'successPage' => $successPage, 'format' => $outputFormat), 3600);
		
		$runData->contextAdd("fkey", $key);
	}
	
}
