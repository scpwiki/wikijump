<?php

namespace Ozone\Framework;



use Wikidot\Utils\GlobalProperties;
use Wikijump\Helpers\LegacyTools;

/**
 * Flow controller for AJAX requests.
 *
 */
class AjaxModuleWebFlowController extends WebFlowController {

	public function process() {
		global $timeStart;

		// initialize logging service
		$logger = OzoneLogger::instance();
		$loggerFileOutput = new OzoneLoggerFileOutput();
		$loggerFileOutput->setLogFileName(WIKIJUMP_ROOT."/logs/ozone.log");
		$logger->addLoggerOutput($loggerFileOutput);

		$logger->debug("AJAX module request processing started, logger initialized");

		Ozone ::init();

		$runData = new RunData();
		/* processing an AJAX request! */
		$runData->setAjaxMode(true);

		$runData->init();

		// Extra return array - just for ajax handling
		$runData->ajaxResponseAdd("status", "OK");

		Ozone :: setRunData($runData);
		$logger->debug("RunData object created and initialized");

		// handle session at the begging of procession
		$runData->handleSessionStart();

		$template = $runData->getModuleTemplate();
		$classFile = $runData->getModuleClassPath();
		$class = LegacyTools::getNamespacedClassFromPath($runData->getModuleClassPath());
		$logger->debug("processing template: ".$runData->getModuleTemplate().", Class: $class");

		require_once ($classFile);
		$module = new $class();

		// module security check
		if(!$module->isAllowed($runData)){
			if($classFile == $runData->getModuleClassPath()){
				$runData->setModuleTemplate("errors/NotAllowed");
			} else {
				// $module->isAllowed() should set the error template!!! if not -
				// default NotAllowed is used

				// reload the Class again - we do not want the unsecure module to render!
				$classFile = $runData->getModuleClassPath();
                $class = LegacyTools::getNamespacedClassFromPath($runData->getModuleClassPath());
				$logger->debug("processing template: ".$runData->getModuleTemplate().", Class: $class");
				require_once ($classFile);
				$module = new $class();
				$runData->setAction(null);
			}
		}

		Ozone::initSmarty();
		$logger->debug("OZONE initialized");

		Ozone :: initServices();
		$logger->debug("Smarty template services loaded");
		Ozone :: parseMacros();
		$logger->debug("Smarty macros parsed");
		Ozone :: updateSmartyPlain();
		$logger->debug("plain version of Smarty created");

		$logger->info("Ozone engines successfully initialized");

		// PROCESS ACTION

		$actionClass = $runData->getAction();
		$logger->debug("processing action $actionClass");
		while ($actionClass != null) {

			require_once (PathManager :: actionClass($actionClass));
            $class = LegacyTools::getNamespacedClassFromPath(PathManager :: actionClass($actionClass));
			$action = new $class();

			// action security check
			$classFile = $runData->getModuleClassPath();
			if(!$action->isAllowed($runData)){
				if($classFile == $runData->getModuleClassPath()){
					$runData->setModuleTemplate("errors/NotAllowed");
				}
				// $action->isAllowed() should set the error template!!! if not -
				// default NotAllowed is used
				break;

			}

			$actionEvent = $runData->getActionEvent();
			if ($actionEvent != null) {
				$action-> $actionEvent ($runData);
				$logger->debug("processing action: $actionClass, event: $actionEvent");
			} else {
				$logger->debug("processing action: $actionClass");
				$action->perform($runData);
			}
			// this is in case action changes the action name so that
			// the next action can be executed.
			if ($runData->getNextAction() != null) {
				$actionClass = $runData->getNextAction();
				$runData->setAction($actionClass);
				$runData->setActionEvent($runData->getNextActionEvent());
			} else {
				$actionClass = null;
			}
		}

		// end action process

		// check if template has been changed by the module. if so...
		if($template != $runData->getModuleTemplate()){
			$classFile = $runData->getModuleClassPath();
			$class = LegacyTools::getNamespacedClassFromPath($runData->getModuleClassPath());
			$logger->debug("processing template: ".$runData->getModuleTemplate().", Class: $class");

			require_once ($classFile);
			$module = new $class();
		}

		$module->setTemplate($template);
		$rendered = $module->render($runData);

		$rVars = $runData->getAjaxResponse();

		if ($rendered != null) {
			// process modules...
	 		$moduleProcessor = new ModuleProcessor($runData);
	 		$out = $moduleProcessor->process($rendered);
	 		$rVars['body'] = $out;
		}
		$json = new JSONService();
 		$out = $json->encode($rVars);
		echo $out;

		$runData->handleSessionEnd();

	}

}
