<?php

namespace Ozone\Framework;


use Smarty;
use Wikidot\Utils\GlobalProperties;

define('SMARTY_DIR', WIKIJUMP_ROOT.'/lib/smarty/libs/');
require_once (SMARTY_DIR.'Smarty.class.php');

/**
 * Wrapper for the Smarty Class.
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
		//Extra dir for application extensions
		$this->plugins_dir[] = WIKIJUMP_ROOT.'/php/Smarty/Plugins/';

		$this->load_filter('pre', 'defmacrohelp');

		$this->assign("URL_HOST", GlobalProperties::$URL_HOST);
		$this->assign("URL_DOMAIN", GlobalProperties::$URL_DOMAIN);
		$this->assign("URL_DOCS", GlobalProperties::$URL_DOCS);
		$this->assign("HTTP_SCHEMA", GlobalProperties::$HTTP_SCHEMA);
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
