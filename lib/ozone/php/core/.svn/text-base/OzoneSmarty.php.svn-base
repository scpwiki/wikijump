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

define('SMARTY_DIR', WIKIDOT_ROOT.'/lib/smarty/libs/');
require_once (SMARTY_DIR.'Smarty.class.php');

/**
 * Wrapper for the Smarty class.
 *
 */
class OzoneSmarty extends Smarty{

	private $currentTemplate;
	private $macros = array();
##	public $plugins_dir;
	
	public function __construct(){
			
		$this->compiler_file = OZONE_ROOT.'/php/core/OzoneSmartyCompiler.php';
		$this->compiler_class = 'OzoneSmartyCompiler'; 
		
		$this->compile_dir = PathManager::smartyCompileDir();		
		$this->cache_dir = PathManager::smartyCacheDir();

		$this->plugins_dir = array(PathManager::smartyPluginDir(), PathManager::smartyOzonePluginDir());
		//extra dir for application extensions
		$this->plugins_dir[] = WIKIDOT_ROOT.'/php/smarty_plugins/'; 
		
		$this->load_filter('pre', 'defmacrohelp');	
		
		$this->assign("URL_HOST", GlobalProperties::$URL_HOST);
		$this->assign("URL_DOMAIN", GlobalProperties::$URL_DOMAIN);
		$this->assign("URL_DOCS", GlobalProperties::$URL_DOCS);
		$this->assign("IP_HOST", GlobalProperties::$IP_HOST);		
		$this->assign("SERVICE_NAME", GlobalProperties::$SERVICE_NAME);		
		$this->assign("SUPPORT_EMAIL", GlobalProperties::$SUPPORT_EMAIL);		
	}	
	
	public function fetch($resource_name, $cache_id = null, $compile_id = null, $display = false){
		$this->currentTemplate = $resource_name;
		return parent::fetch($resource_name, $cache_id, $compile_id, $display);
	}
	
	public function getCurrentTemplate(){
		return $this->currentTemplate;	
	} 
	
	public function registerMacro($name, $template){
		$this->macros[$name]=$template;	
	}
	
	public function getMacroTemplateFileName($name){
		return $this->macros[$name];	
	}
	
	public function getMacroRegister(){
		return $this->macros;	
	}
	
	public function setMacroRegister($macroRegister){
		$this->macros = $macroRegister;	
	}
	
	public function getTemplateVars(){
		return $this->_tpl_vars;	
	}
	
	public function setTemplateVars($vars){
		$this->_tpl_vars = $vars;	
	}
	
}
