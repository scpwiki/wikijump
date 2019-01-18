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
 * @package Ozone
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */
 
/**
 * Path manager.
 *
 */
class PathManager{

	public static function ozonePhpCoreFile($fileName){
		return OZONE_ROOT."/php/core/".$filename;	
	}	
	
	public static function ozonePhpServiceFile($fileName){
		return OZONE_ROOT."/php/resource/".$filename;	
	}
	
	public static function ozonePhpServiceAutoloadDir(){
		return OZONE_ROOT."/php/template_services/autoload/";	
	}
	
	public static function ozoneApplicationPhpServiceAutoloadDir(){
		return WIKIDOT_ROOT."/php/template_services/autoload/";	
	}
	
	public static function ozonePhpServiceOnDemandDir(){
		return OZONE_ROOT."/php/template_services/ondemand/";	
	}
	
	public static function ozoneApplicationPhpServiceOnDemandDir(){
		return WIKIDOT_ROOT."/php/template_services/ondemand/";	
	}
	
	public static function smartyDir(){
		return 	WIKIDOT_ROOT."/lib/smarty/libs/";
	}
	
	public static function smartyOzonePluginDir(){
		return 	OZONE_ROOT."/php/smarty_plugins/";
	}
	
	public static function smartyPluginDir(){
		return 	WIKIDOT_ROOT."/lib/smarty/libs/plugins/";
	}
	
	public static function smartyApplicationPluginDir(){
		return 	WIKIDOT_ROOT."/php/smarty_plugins/";
	}
	
	public static function smartyCompileDir(){
			return 	WIKIDOT_ROOT."/tmp/smarty_templates_c/";
	} 
	
	public static function smartyCacheDir(){
			return 	WIKIDOT_ROOT."/tmp/smarty_cache/";
	} 
	
	public static function smartyMacroTemplateDir(){
			return 	WIKIDOT_ROOT."/tmp/smarty_macro_templates/";
	}

	public static function navigationTemplate($name){
		
		return WIKIDOT_ROOT."/templates/navigations/".$name.".tpl";	
	}
	
	public static function navigationTemplateDir(){
		
		return WIKIDOT_ROOT."/templates/navigations/";	
	}
	
	public static function layoutTemplate($name){
		return 	WIKIDOT_ROOT."/templates/layouts/".$name.".tpl";
	}
	
	public static function screenTemplate($name){
		str_replace(',', '/', $name);
		return 	WIKIDOT_ROOT."/templates/screens/".$name.".tpl";
	}
	
	public static function templateDir(){
		return 	WIKIDOT_ROOT."/templates/";
	}
	
	public static function screenClass($className){
		$className = str_replace(',', '/', $className);
		return 	WIKIDOT_ROOT."/php/screens/".$className.".php";
	}
	
	public static function screenClassDir(){
		return 	WIKIDOT_ROOT."/php/screens/";
	}

	public static function actionClass($className){
		return 	WIKIDOT_ROOT."/php/actions/".$className.".php";
	}
	
	public static function moduleTemplate($name){
		return 	WIKIDOT_ROOT."/templates/modules/".$name.".tpl";
	}
	
	public static function moduleClass($className){
		return 	WIKIDOT_ROOT."/php/modules/".$className.".php";
	}
	
	public static function moduleClassDir(){
		return 	WIKIDOT_ROOT."/php/modules/";
	}
	
	public static function macroDir(){
		return 	WIKIDOT_ROOT."/templates/macros/";
	}
	
	public static function macroFile($name){
		return 	WIKIDOT_ROOT."/templates/macros/".$name.".tpl";
	}
	
	public static function dbClass($className){
		return 	WIKIDOT_ROOT."/php/db/".$className.".php";
	}
	
	public static function formSpecFile($formName){
		return 	WIKIDOT_ROOT."/forms/".$formName."-form.xml";
	}
	
	public static function listSpecFile($listName){
		return 	WIKIDOT_ROOT."/forms/".$listName."-list.xml";
	}
	
	public static function externalLibFile($fileName){
		return WIKIDOT_ROOT.'/lib/'.$fileName;
	}
	
	public static function emailTemplate($templateName){
		return 	WIKIDOT_ROOT."/templates/emails/".$templateName.".tpl";
	}
	
	public static function fileUploadDir(){
			return 	WIKIDOT_ROOT."/tmp/file_uploads/";
	}
	
	public static function messagesDir(){
		return WIKIDOT_ROOT."/messages/";	
	}
}
