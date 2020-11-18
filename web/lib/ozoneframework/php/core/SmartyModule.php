<?php



/**
 * Abstract class for smarty-based modules.
 *
 */
abstract class SmartyModule extends Module{

	private $template;

	public function render($runData){

	 	if($runData->getModuleTemplate() == null){return;}

	 	$this->build($runData);

	 	$template = $runData->getModuleTemplate();
	 	$templateFile  = PathManager::moduleTemplate($template);
	 	// render!

	 	$smarty = Ozone::getSmartyPlain();

	 	$page = $runData->getPage();
	 	$smarty->assign("page", $page);

	 	// put context into context

	 	$context = $runData->getContext();
	 	if($context !== null){
	 		foreach($context as $key => $value){
		 		$smarty->assign($key, $value);
	 		}
	 	}

	 	// put errorMessages and messages into the smarty's context as well.
	 	$dataMessages = $runData->getMessages();
	 	$dataErrorMessages = $runData->getErrorMessages();
	 	if(count($dataMessages) > 0) {
	 		$smarty->assign('data_messages', $dataMessages);
	 	}

	 	if(count($dataErrorMessages) > 0) {
	 		$smarty->assign('data_errorMessages', $dataErrorMessages);
	 	}

	 	$out = $smarty->fetch($templateFile);

	 	return $out;

	 }

	 public function setTemplate($template){
	 	$this->template = $template;
	 }

	 public function getTemplate(){
	 	return $this->template;
	 }

	/**
	 * builds context
	 */
	abstract public function build($runData);

}
