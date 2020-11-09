<?php



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

	public function getService($className){
		if(isset($this->storage["$className"])){
			return 	$this->storage["$className"];
		} else {
			require_once PathManager::ozonePhpServiceOnDemandDir().$className.".php";
			$instance = new $className($this->runData);
			$this->storage["$className"] = $instance;
			return $instance;
		}
	}

}
