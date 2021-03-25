<?php

namespace Ozone\Framework\Template\Services\Autoload;



use Ozone\Framework\PathManager;
use Ozone\Framework\TemplateService;
use Wikijump\Helpers\LegacyTools;

/**
 * TemplateServiceManager is a TemplateService for accessing
 * on-demand (not autoloaded) services.
 */
class TemplateServiceManager extends TemplateService{
	protected $serviceName = "serviceManager";
	private $runData;
	protected $storage = array();

	public function __construct($runData){
		$this->runData = $runData;
	}

	public function getService($class){
		if(isset($this->storage["$class"])){
			return 	$this->storage["$class"];
		} else {
			require_once PathManager::ozonePhpServiceOnDemandDir().$class.".php";
			$class = LegacyTools::getNamespacedClassFromPath(PathManager::ozonePhpServiceOnDemandDir().$class.".php");
			$instance = new $class($this->runData);
			$this->storage["$class"] = $instance;
			return $instance;
		}
	}

}
