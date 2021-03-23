<?php

namespace Ozone\Framework\Template\Services\Autoload;



use Ozone\Framework\TemplateService;
use Wikidot\Utils\GlobalProperties;

/**
 * UI manager service.
 *
 */
class UIManager extends TemplateService{

	protected $serviceName = "ui";

	/** Just a cache of PageProperties object */
	private $page;

	public function __construct($runData = null){
		$this->page = $runData->getPage();
	}

	public function getBaseURL(){
		return GlobalProperties::$HTTP_SCHEMA . "://" . GlobalProperties::$URL_HOST."/";
	}

	/**
	 * Returns full URL for the given CSS filename.
	 * @param string $filename
	 * @return string full URL
	 */
	public function style($filename){
		return GlobalProperties::$HTTP_SCHEMA . "://" . GlobalProperties::$URL_HOST."/ui/skins/". $this->page->getSkin()."/css/".$filename;
	}

	/**
	 * Returns full URL for the given JavaScript filename.
	 * @param string $filename
	 * @return string full URL
	 */
	public function javaScript($filename){
		return GlobalProperties::$HTTP_SCHEMA . "://" . GlobalProperties::$URL_HOST."/ui/skins/". $this->page->getSkin()."/js/".$filename;
	}

	/**
	 * Returns full URL for the given image filename.
	 * @param string $filename
	 * @return string full URL
	 */
	public function image($filename){
		return GlobalProperties::$HTTP_SCHEMA . "://" . GlobalProperties::$URL_HOST."/ui/skins/". $this->page->getSkin()."/images/".$filename;
	}

	public function getImageBaseURL(){
		return GlobalProperties::$HTTP_SCHEMA . "://" . GlobalProperties::$URL_HOST."/ui/skins/". $this->page->getSkin()."/images/";
	}

}
