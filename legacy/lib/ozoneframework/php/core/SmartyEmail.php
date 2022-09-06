<?php

namespace Ozone\Framework;



/**
 * Email Class that uses Smarty and PHPMailer.
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
