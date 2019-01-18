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
 * Core object for the OZONE Framework.
 *
 */
class Ozone {

	public static $myConnection;
	public static $smarty;
	public static $smartyPlain;
	public static $smartyInitialized = false;
	
	public static $memcache = null;
	
	public static $smartyPlainTemplateVars;

	public static $runData;

	public static function init() {
		// create db connection
		
		// see if tmp directories exist - and create if not
		
		$dir = PathManager::smartyCompileDir();
		if(!file_exists($dir)) { mkdirfull($dir);}
		
		$dir = PathManager::smartyCacheDir();
		if(!file_exists($dir)) { mkdirfull($dir);}
		
		$dir = PathManager::smartyMacroTemplateDir();
		if(!file_exists($dir)) { mkdirfull($dir);}
		
		// connect to memcache server
		if(GlobalProperties::$USE_MEMCACHE == true){
			self::$memcache = new Memcache();
			self::$memcache->connect(GlobalProperties::$MEMCACHE_HOST, GlobalProperties::$MEMCACHE_PORT);
			self::$memcache->setCompressThreshold(5000); 
		}else{
			self::$memcache = new DummyMemcache();
		}

	}
	
	public static function initSmarty($initServices = true){
		 //initialize SMARTY		
		 
		self :: $smarty = new OzoneSmarty();
		self::$smarty->load_filter('pre', 'defmacrohelp');
		if($initServices){
			self::initServices();
		}
		self::parseMacros();
		self::updateSmartyPlain();
		self::$smartyInitialized = true;

	}
	public static function initServices() {
		//load resources to the smarty's context from autoload dir
		$audir = PathManager::ozonePhpServiceAutoloadDir();
		$serviceFiles = ls($audir, "*.php");
		foreach ($serviceFiles as $sf){
			require_once ($audir.$sf);
			$class = str_replace('.php', '', $sf);
			$service = new $class(self::$runData);
			self :: $smarty->assign($service->serviceName(), $service);
		}
		
		// load services from application path
		$audir = PathManager::ozoneApplicationPhpServiceAutoloadDir();
		$serviceFiles = ls($audir, "*.php");
		foreach ($serviceFiles as $sf){
			require_once ($audir.$sf);
			$class = str_replace('.php', '', $sf);
			$service = new $class(self::$runData);
			self :: $smarty->assign($service->serviceName(), $service);
		}
		
	}

	public static function parseMacros() {
		$dir = PathManager::macroDir();
		$files = ls($dir, "*.autoload.tpl");
		foreach ($files as $f) {
			self :: $smarty->fetch($dir.$f);
		}
	}

	public static function getMyConnection() {
		return self :: $dbConnection;
	}

	public static function getSmarty() {
		if(self::$smartyInitialized == false){
			self::initSmarty();	
		}
		return self :: $smarty;
	}
	
	public static function updateSmartyPlain(){
		self::$smartyPlain = new OzoneSmarty();
		self::$smartyPlainTemplateVars = self::$smarty->get_template_vars();
		self::$smartyPlain->setMacroRegister(self::$smarty->getMacroRegister());
	}
	
	public static function getSmartyPlain(){
		if(self::$smartyInitialized == false){
			self::initSmarty();	
		}
		$plain = clone(self::$smartyPlain);
		$plain->setTemplateVars(self::$smartyPlainTemplateVars);
		return $plain;	
	}

	public static function reset() {
		## remove temporary files ... 	
	}

	public static function setRunData($runData) {
		self :: $runData = $runData;
	}

	public static function getRunData() {
		return self :: $runData;
	}
}
