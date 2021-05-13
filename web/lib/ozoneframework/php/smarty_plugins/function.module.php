<?php


/**
 * Module placeholder generator for Smarty.
 */
function smarty_function_module($params, & $smarty) {
	if($params['name'] == '') {
		$smarty->trigger_error("module: missing attribute 'name' for the macro");
		return;
	}
	$templateName = $params['name'];
	$parameters = $params['parameters'] ?? null;

	unset($params['name']);
	// convert params to string key="value"
	foreach($params as $key => $value){
		$parameters.="$key=\"$value\" ";
	}

	if($parameters!==null){
		$parmstring = " ".urlencode($parameters)." ";
	}
	else { $parmstring = null; }
	$d = utf8_encode("\xFE");
	$out = $d."module \"".$templateName."\" ".$parmstring.$d;
	return $out;

}
