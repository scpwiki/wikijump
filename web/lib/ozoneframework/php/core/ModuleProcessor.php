<?php

namespace Ozone\Framework;



use Wikidot\Utils\GlobalProperties;
use Wikidot\Utils\ProcessExceptionHandler;
use Wikidot\Utils\WDPermissionException;
use Exception;
use Wikijump\Helpers\LegacyTools;

/**
 * Module processing tool.
 *
 */
class ModuleProcessor {

	private $runData;
	private $javascriptInline = false;
	private $cssInline = false;
	private $cssInclude = array();
	private $modulesToProcessPage = null;

	private $_moduleChain = array();

	private $level = 0;

	public function __construct($runData){
		$this->runData = $runData;
	}

	public function setJavascriptInline($val){
		$this->javascriptInline = $val;
	}

	public function setCssInline($val){
		$this->cssInline = $val;
	}

	public function process($content){
	    if($this->level == 0){
	        $this->_moduleChain = array();
	    }
		$this->level++;
		// search content for some pattern and call the process...
        $d = utf8_encode("\xFE");
        $out = preg_replace_callback("/
            ${d}
            module                  # Module declaration
            \s                      # Whitespace after module declaration
            \"([a-zA-Z0-9\/_]+?)\"  # Module name, wrapped in quotes
            ([^${d}]+?)?            # Match anything else up to end
            ${d}                    # End of module
            /x",
        array(&$this, 'renderModule1'), $content);

		// insert css files if necessary
		if($this->cssInline && count($this->cssInclude) > 0){
			$cstring = 	"";
			foreach($this->cssInclude as $cssin){
				$cstring .= "@import url($cssin);\n";
			}
			// now find a place and insert $cstring!!!
            $regexp = "/
                (
                    <style(?:.*?)           # Match a style tag
                    id=\"internal-style\">  # Assert that it has this ID
                )
                (.*?)                       # Followed by anything else
                (<\/style>)                 # Closing tag
                /sx";
            # Literally why would you do that this way
			$replace = "\\1 \n $cstring \n \\2 \\3";
			/**
             * Blade: This method of regex won't work as the base template isn't
             * passed through, so we'll inject this string if needed.
             */
			$before_out = $out;
			$out = preg_replace($regexp, $replace, $out, 1);

			/**
             * If no change was made, inject into the output.
             */
			if($before_out == $out) {
			    $out = "<style>\n$cstring\n</style>" . $out;
            }
		}

		// TODO: check if top-level?
		if($this->modulesToProcessPage != null){
			$runData = $this->runData;
			foreach($this->modulesToProcessPage as $module){
				$out = $module->processPage($out, $runData);

			}
		}

		return $out;
	}

	public function renderModule1($matches){
		try{
			$out = $this->renderModule($matches[1], $matches[2]);
		}catch(Exception $e){
			$p = new ProcessExceptionHandler();
			$out = $p->handleInlineModule($e, OZONE::getRunData());
		}
		return $out;
	}

	public function renderModule($templateName, $parameters=null){

		$ttt = ModuleHelpers::findModuleClass($templateName);
		$classPath = $ttt[1];
		require_once($classPath);
		$class = LegacyTools::getNamespacedClassFromPath($classPath);
		$moduleClass = new $class();
		$moduleClass->setModuleChain($this->_moduleChain);
		$runData = $this->runData;
		$runData->setModuleTemplate($templateName);

		if(!$moduleClass->isAllowed($runData)){
			throw new WDPermissionException("Not allowed.");
		}

		$pl = $runData->getParameterList();
		$plOrig = clone($pl);
		$contextOrig = $runData->getContext();
		$runData->contextDel();
		// add new parameters ...

		$origParms = clone($runData->getParameterList());
		if($parameters !== null && $parameters !== ""){
			// first parse the parameters string
			$parameters = urldecode($parameters);
			$re = '/([a-zA-Z][a-zA-Z0-9\-_]*)="([^"]*)"/';
			preg_match_all($re, $parameters, $pout,PREG_SET_ORDER);

			for($i = 0; $i<count($pout); $i++){
				if($pout[$i][1] == "module_body"){
					$pout[$i][2] = urldecode($pout[$i][2]);
				}
				$pl->addParameter($pout[$i][1], $pout[$i][2], "MODULE");
			}
		}
		// the RunData object MUST be cloned at this point - to avoid context mixing between
	 	// the module and the global (screen) context.

	 	try{
			$out = $moduleClass->render($runData);
	 	}catch(Exception $e){
	 		// restore old parameters ...
			$runData->setContext($contextOrig);
			$runData->setParameterList($plOrig);

			// recurent (for nested modules to work):
			$out = $this->process($out);
			$this->level--;
			throw $e;
	 	}

		// check if there are any javascript files for this module

		$js2include = array();

		$file = WIKIJUMP_ROOT.'/'.GlobalProperties::$MODULES_JS_PATH.'/'.$templateName.'.js';
		if(file_exists($file)){
			$url = 	GlobalProperties::$MODULES_JS_URL.'/'.$templateName.'.js';
			$js2include[] = $url;
		}
		$js2include = array_merge($js2include, $moduleClass->getExtraJs());
		foreach($js2include as $jsUri){
			if($this->javascriptInline){
				// and include them via <script> tags now
				$incl = '<script type="text/javascript" src="'.$jsUri.'"></script>';
				$out .= $incl;
			} else {
				// include later
				$jsInclude = $runData->getTemp("jsInclude");
				$jsInclude[] = $jsUri;
				$runData->setTemp("jsInclude", $jsInclude);
			}
		}

		// check if any css file exists

		$file = WIKIJUMP_ROOT.'/'.GlobalProperties::$MODULES_CSS_PATH.'/'.$templateName.'.css';
		if(file_exists($file)){
			$url = 	GlobalProperties::$MODULES_CSS_URL.'/'.$templateName.'.css';
			$this->cssInclude[] = $url;
		}

		// restore old parameters ...
		$runData->setContext($contextOrig);
		$runData->setParameterList($plOrig);

		$moduleChainOrig = array_copy($this->_moduleChain);
		$this->_moduleChain[] = $moduleClass;

		// recurent (for nested modules to work):
		$out = $this->process($out);
		$this->level = $this->level - 1;
		$this->_moduleChain = $moduleChainOrig;
		// check if the module wants to modify the page itself
		if($moduleClass->getProcessPage()){
			$this->modulesToProcessPage[] = $moduleClass;
		}

		return $out;

	}

}
