<?php

namespace Ozone\Framework;



use Wikidot\Utils\GlobalProperties;

/**
 * Default email Class.
 *
 */
class OzoneEmail extends SmartyEmail{

	protected $_toAddresses = array();

	public function __construct(){
		parent::__construct();
		// set default values
		$this->setSMTP(); // telling the Class to use SMTP
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
