<?php



/**
 * Macro loader.
 *
 */
class MacroLoaderService extends TemplateService {

	protected $serviceName = "macros";

	private $macroPath;

	function __construct($runData = null){
			$this->macroPath = PathManager::macroDir();
	}

	public function load($macroSet){
		$smarty = Ozone::getSmarty();

		$smarty->fetch($this->macroPath . $macroSet.'.tpl');

		// should we load it for the plain smarty too?
	}

}
