<?php

namespace Ozone\Framework\Template\Services\Autoload;



use Ozone\Framework\TemplateService;

/**
 * Global variables service for Smarty.
 */
class GlobalVariablesService extends TemplateService {

	protected $serviceName = "globals";

	private $storage = array();
	private $runData;

	public function __construct($runData){
		$this->runData = $runData;
	}

	public function set($key, $value){
		$this->storage[$key] = $value;
	}

	public function del($key = null){
		if($key !== null){
			unset($this->storage[$key]);
		} else {
			$this->storage = array();
		}
	}

	public function get($key){
		return $this->storage[$key];
	}

	public function hasKey($key){
		if($this->storage[$key] !== null){
			return true;
		}	else {
			return false;
		}
	}

}
