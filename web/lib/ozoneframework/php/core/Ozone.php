<?php

namespace Ozone\Framework;



use Wikidot\Utils\GlobalProperties;
use Memcache;
use Wikijump\Helpers\LegacyTools;

/**
 * Core object for the OZONE Framework.
 *
 */
class Ozone {

	public static $myConnection;
	public static $smarty;
	public static $smartyPlain;
	public static $smartyInitialized = false;

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
			$class = LegacyTools::getNamespacedClassFromPath($audir.$sf);
			$service = new $class(self::$runData);
			self :: $smarty->assign($service->serviceName(), $service);
		}

		// load services from application path
		$audir = PathManager::ozoneApplicationPhpServiceAutoloadDir();
		$serviceFiles = ls($audir, "*.php");
		foreach ($serviceFiles as $sf){
			require_once ($audir.$sf);
            $class = LegacyTools::getNamespacedClassFromPath($audir.$sf);
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
