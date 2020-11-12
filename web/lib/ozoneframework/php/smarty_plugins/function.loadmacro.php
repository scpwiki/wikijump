<?php


/**
 * Macro loader for Smarty.
 */
function smarty_function_loadmacro($params, & $smarty) {
	if($params['set'] == '') {
		$smarty->trigger_error("macro: missing attribute 'set' for the loadmacro");
		return;
	}

	$macroPath = PathManager::macroDir();
	$smarty->fetch($macroPath . $params['set'].'.tpl');
}
