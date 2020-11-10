<?php


/**
 * Macro calling method for Smarty.
 */
function smarty_function_macro($params, & $smarty) {
	if($params['name'] == '') {
		$smarty->trigger_error("macro: missing attribute 'name' for the macro");
		return;
	}

	## get macro file name
	$templateFilename = $smarty->getMacroTemplateFileName($params['name']);

	if($templateFilename == null) {
		$smarty->trigger_error("macro: template file for the macro missing");
		return;
	}

	// get new smarty instance to process the template:
	$subSmarty = Ozone::getSmartyPlain();
	unset($params['name']);

	$subSmarty->assign('params',$params);
	foreach($params as $key => $value){
		$subSmarty->assign($key, $value);
	}

	## copy the macro register
	$subSmarty->setMacroRegister($smarty->getMacroRegister());

	#render the content
	$out = $subSmarty->fetch(PathManager::smartyMacroTemplateDir()."/".	$templateFilename);
	return $out;
}
