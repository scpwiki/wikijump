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
 * UI manager service.
 *
 */
class UIManager extends TemplateService{
	
	protected $serviceName = "ui";

	/** Just a cache of PageProperties object */
	private $page;
	
	public function __construct($runData = null){
		$this->page = $runData->getPage();
	}

	public function getBaseURL(){
		return "http://".GlobalProperties::$URL_HOST."/";	
	}

	/**
	 * Returns full URL for the given CSS filename. 
	 * @param string $filename
	 * @return string full URL
	 */
	public function style($filename){
		return "http://".GlobalProperties::$URL_HOST."/ui/skins/". $this->page->getSkin()."/css/".$filename;	
	}
	
	/**
	 * Returns full URL for the given JavaScript filename.
	 * @param string $filename
	 * @return string full URL
	 */
	public function javaScript($filename){
		return "http://".GlobalProperties::$URL_HOST."/ui/skins/". $this->page->getSkin()."/js/".$filename;
	}
	
	/**
	 * Returns full URL for the given image filename.
	 * @param string $filename
	 * @return string full URL
	 */
	public function image($filename){
		return "http://".GlobalProperties::$URL_HOST."/ui/skins/". $this->page->getSkin()."/images/".$filename;
	}
	
	public function getImageBaseURL(){
		return "http://".GlobalProperties::$URL_HOST."/ui/skins/". $this->page->getSkin()."/images/";
	}
	
}
