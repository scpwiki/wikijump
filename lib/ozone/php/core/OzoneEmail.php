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
 * Default email class.
 *
 */
class OzoneEmail extends SmartyEmail{
	
	protected $_toAddresses = array();

	public function __construct(){
		parent::__construct();
		// set default values
		$this->setSMTP(); // telling the class to use SMTP
		$this->setHost(GlobalProperties::$DEFAULT_SMTP_HOST); // SMTP server
		if(GlobalProperties::$DEFAULT_SMTP_AUTH){
			$this->setSMTPAuth(GlobalProperties::$DEFAULT_SMTP_AUTH);
			$this->setUsername(GlobalProperties::$DEFAULT_SMTP_USER);
			$this->setPassword(GlobalProperties::$DEFAULT_SMTP_PASSWORD);
			$this->setSMTPSecure(GlobalProperties::$DEFAULT_SMTP_SECURE);
		}
		$this->setPort(GlobalProperties::$DEFAULT_SMTP_PORT);
		
		$this->setHostname(GlobalProperties::$DEFAULT_SMTP_HOSTNAME);
		$this->setFrom(GlobalProperties::$DEFAULT_SMTP_FROM_EMAIL);
		$this->setFromName(GlobalProperties::$DEFAULT_SMTP_FROM_NAME);
		$this->setCharSet("UTF-8");
		$this->setEncoding("quoted-printable"); 
		
		if(GlobalProperties::$DEFAULT_SMTP_SENDER){
			$this->setSender(GlobalProperties::$DEFAULT_SMTP_SENDER);
		}
	}
	
	public function addAddress($address, $name=""){
		parent::addAddress($address, $name);	
		$this->_toAddresses[] = $address;
	}
	
}
