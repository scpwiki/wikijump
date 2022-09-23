<?php

namespace Ozone\Framework;


use Wikidot\Utils\GlobalProperties;

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
