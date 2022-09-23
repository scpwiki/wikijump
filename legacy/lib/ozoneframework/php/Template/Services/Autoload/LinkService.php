<?php

namespace Ozone\Framework\Template\Services\Autoload;



use Ozone\Framework\TemplateService;
use Wikidot\Utils\GlobalProperties;

/**
 * Link-building service.
 */
class LinkService extends TemplateService{

	protected $serviceName = "link";

	private $protocol = "http";
	private $queryPath;
	private $parameters = array ();
	private $templateName;

	private $redirect = false;

	private $runData;

	private $languageCache;
	private $skinCache;

	private $passLanguage = false;
	private $passSkin = false;

	public function __construct($runData = null){
		$this->languageCache = $runData->getLanguage();
		if($this->languageCache != GlobalProperties::$DEFAULT_LANGUAGE){
			$this->passLanguage=true;
		}
		$this->skinCache = $runData->getPage()->getSkin();
		if($this->skinCache != GlobalProperties::$DEFAULT_SKIN){
			$this->passSkin = true;
		}

		$this->runData = $runData;
	}

	public function getProtocol() {
		return $this->protocol;
	}

	public function setProtocol($protocol) {
		$this->protocol = $protocol;
	}

	public function setTemplate($templateName) {
		$this->templateName = $templateName;
		return $this;
	}

	public function addParameter($name, $value) {
		$this->parameters[$name] = $value;
		return $this;
	}

	public function addParameters($parameterArray) {
		$this->parameters = array_merge($this->parameters, $parameterArray);
		return $this;
	}

	public function setAction($actionName){
		$this->parameters['action'] = $actionName;
		return $this;
	}

	public function clearParameter($name) {
		unset ($this->parameters[$name]);
		return $this;
	}

        public function delParameter($name) {
	       unset ($this->parameters[$name]);
	       return $this;
	}

	public function render($noEscape=false) {

		// if using the redirection...
		if($this->redirect == true){
			// catch the full url
			$this->redirect = false;
			$url = 	$this->render(); //should noEscape be true?
			$out = $this->protocol."://".GlobalProperties::$URL_HOST;
			$out .= "/InstantRedirect/redirect_url/".urlencode($url);
			return $out;
		}

		// check if to pass language and skin
		if($this->passLanguage){
			$this->addParameter("lang", $this->languageCache );
		}
		if($this->passSkin){
			$this->addParameter("skin", $this->skinCache);
		}

		$out = $this->protocol."://".GlobalProperties::$URL_HOST;

		// sort the array...
		ksort($this->parameters);

		if (isset ($this->templateName)) {
		  $template =  $this->templateName;
		  $template = str_replace("/", "__", $template);
		  $this->parameters =  array("template" => $template) + $this->parameters;
		} else {
			// template should ALWAYS be present!!!
			$this->parameters =  array("template" => "Index") + $this->parameters;
		}

		$ps = '';

		// with mod_rewrite...

		//first the template:

		$ps.="/".$this->parameters['template'];
		unset($this->parameters['template']);
		foreach($this->parameters as $key => $value){
			if($key != null && $value != null){
				if($key == "action"){
					$value = 	str_replace("/", "__", $value);
				}
			  	$ps	.= "/$key/".urlencode($value);
			}
		}

		//
		$out.=$ps;

		//clear everything now!!!
		$this->resetAll();

		return $out;

	}

	public function renderNoModRewrite($noEscape=false){

	}

	public function renderModRewrite($noEscape=false){

	}

	public function __toString(){
		return $this->render();
	}

	public function copy(){
		return clone($this);
	}

	public function resetAll(){
		$this->parameters = array ();
		$this->protocol = "http";
		$this->templateName = null;
		return $this;
	}

	public function setSecure($value){
		$this->protocol = "https";
		return $this;
	}

	public function setRedirect($value){
		$this->redirect = $value;
		return $this;
	}

}
