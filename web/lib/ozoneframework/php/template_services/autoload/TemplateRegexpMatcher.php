<?php



/**
 * Service for matching the current template name against given regular
 * expression. Should be useful when making menus etc.
 */
class TemplateRegexpMatcher extends TemplateService{

	private $runData;
	protected $serviceName = "templateMatcher";

	public function __construct($runData){
		$this->runData = $runData;
	}

	public function match($pattern){
		return preg_match("/".$pattern."/", $this->runData->getScreenTemplate());
	}

}
