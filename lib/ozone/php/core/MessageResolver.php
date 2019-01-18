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
 * Message resolver.
 *
 */
class MessageResolver {
	
	public static $resolver;
	
	private $messagesXML;
	
	public static function instance(){
		if(self::$resolver == null){
			self::$resolver = new MessageResolver();
		}
		return self::$resolver;	
	}
	
	protected function loadMessages(){
		$dir = PathManager::messagesDir();
		$file = $dir.'/messages.xml';
		$xml = simplexml_load_file($file);
		$this->messagesXML = $xml;
	}
	
	public function message($key, $lang=null){
		if($lang == null){
			$lang = Ozone::$runData->getLanguage();	
		}
		if($this->messagesXML == false){
			$this->loadMessages();
		}
		$xml = $this->messagesXML;
		//get the message now!!!
		$res = $xml->xpath("/messages/message[@key='$key']/text[@lang='$lang']");
		$res =  "$res[0]";	
		if($res == null){
			//fall back to the default language
			$res = $xml->xpath("/messages/message[@key='$key']/text[@lang='".GlobalProperties::$DEFAULT_LANGUAGE."']");
			$res =  "$res[0]";	
		}
		if($res == null){
			$res = $xml->xpath("/messages/message[@key='$key']/text[1]");
			$res =  "$res[0]";	
		}
		return $res;
	} 
	
}
