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
 * Abstract class for the web flow modules.
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
