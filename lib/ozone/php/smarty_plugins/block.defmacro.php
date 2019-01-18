<?php
/**
 * Wikidot - free wiki collaboration software
 * Copyright (c) 2008, Wikidot Inc.
 * 
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * For more information about licensing visit:
 * http://www.wikidot.org/license
 * 
 * @category Ozone
 * @package Ozone_Web
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */
 
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
