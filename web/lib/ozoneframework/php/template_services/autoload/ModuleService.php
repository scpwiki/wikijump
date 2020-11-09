<?php



/**
 * Module service.
 *
 */
class ModuleService extends TemplateService {

	protected $serviceName = "module";

	private $templateName;
	private $runData;

	public function __construct($runData){
		$this->runData = $runData;
	}

	public function render($templateName, $parameters=null){
		$this->templateName = $templateName;
		if($parameters!==null){
			$parmstring = " ".urlencode($parameters)." ";
		}
		$d = utf8_encode("\xFE");
		$out = $d."module \"".$templateName."\" ".$parmstring.$d;
		return $out;

	}

	public function __toString() : string {

	}

}
