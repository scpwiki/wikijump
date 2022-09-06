<?php

namespace Ozone\Framework;





/**
 * Abstract Class for the web flow modules.
 *
 */
abstract class Module {

	/**
	 * Describes if contents of the whole page should be processed by
	 * this module. If yes - a method processPage will be called.
	 */
	protected $processPage = false;

	protected $extraJs = array();
	protected $extraCss = array();

	protected $includeDefaultJs = true;
	protected $includeDefaultCss = true;

	protected $_moduleChain = array();

	public function isAllowed($runData){
		return true;
	}

	abstract public function render($runData);

	public function getProcessPage(){
	 	return $this->processPage;
	 }

	 /**
	  * Override this method if you want to process the whole page too.
	  */
	 public function processPage($content, $runData){
	 	return $content;
	 }

	 public function getExtraJs(){
		return $this->extraJs;
	 }

	 public function getExtraCss(){
		return $this->extraCss;
	 }

	 public function setIncludeDefaultJs($val){
	 	$this->includeDefaultJs = $val;
	 }

	 public function getIncludeDefaultJs(){
	 	return 	$this->includeDefaultJs;
	 }

	  public function setIncludeDefaultCss($val){
	 	$this->includeDefaultCss = $val;
	 }

	 public function getIncludeDefaultCss(){
	 	return 	$this->includeDefaultCss;
	 }

	 public function addExtraJs($val){
	 	$this->extraJs[] = $val;
	 }

	 public function setModuleChain(array $moduleChain){
	     $this->_moduleChain = $moduleChain;
	 }
}
