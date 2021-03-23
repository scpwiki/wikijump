<?php

use Ozone\Framework\PathManager;


/**
 * Macro definition block for Smarty.
 */
function smarty_block_defmacro($params, $content, & $smarty, & $repeat) {
	if (isset ($content)) {
		## create a file to store the macro
		if ($params['name'] == '') {
			$smarty->trigger_error("defmacro: unspecified attribute 'name' for the macro");
			return;
		}
		$fileName = $smarty->getCurrentTemplate();
		$templateNameString = str_replace(PathManager::templateDir(), '', $fileName);
		;
		$templateNameString = str_replace('/', ',', $templateNameString);

		$templateNameString .= ','.$params['name'].'.tpl';

		# now copy $content to file $templateNameString

		$fullPath = PathManager::smartyMacroTemplateDir().$templateNameString;
		if (!file_exists($fullPath)) {
			$handle = fopen($fullPath, "w");
			fwrite($handle, $content);
			fclose($handle);
		}
		## ok, now register the macro

		$smarty->registerMacro($params['name'], $templateNameString);

		##echo $templateNameString;
		return '';
	}
}
