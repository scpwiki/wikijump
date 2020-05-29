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
