<?php

namespace Ozone\Framework\Template\Services\Autoload;



use Ozone\Framework\Ozone;
use Ozone\Framework\PathManager;
use Ozone\Framework\TemplateService;

/**
 * Navigation service.
 *
 */
class NavigationService extends TemplateService {

	protected $serviceName = "navigation";

	private $navPath;

	function __construct($runData = null){
			$this->navPath = PathManager::navigationTemplateDir();

	}

	public function render($navigationTemplate){
		$smarty = Ozone::getSmarty();

		return 	$smarty->fetch($this->navPath . $navigationTemplate.'.tpl');
	}

}
