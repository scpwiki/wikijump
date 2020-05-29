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
 * @category Wikidot
 * @package Wikidot
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

/**
 * Module controlls permissions, resolves module names etc.
 */
class ModuleManager {
	
	private static $instance;
	
	private $wikiConfig;
	private $ajaxConfig;
	
	public static function instance(){
		if(self::$instance == null){
			self::$instance = new ModuleManager();	
		}
		return self::$instance;
	}
	
	/**
	 * Takes module name used in wiki source and returns template name
	 */
	public function resolveWikiModuleName($name){
		if($this->wikiConfig == null){
			$this->loadWikiConfig();	
		}
		$row = $this->wikiConfig[$name];
		return $row['template'];
	}

	private function loadWikiConfig(){
	    /* find all files with module configs */
	    $fs = glob(WIKIDOT_ROOT.'/conf/wiki_modules/*.conf');
	    $cont = '';
	    foreach($fs as $f){
	        $c = file_get_contents($f);
	        $c = preg_replace(';^#.*?$;sm', '', $c);
	        $c = trim($c);
	        $cont .= "\n" . $c;
	    }
	    $cont = trim($cont);
		$m1 = explode("\n", $cont);
		$stor = array();
		foreach($m1 as $m){
			$m3 = explode(" ", $m);
			$stor[$m3[0]] = array('name' => $m3[0], 'template' =>$m3[1], 'permissions' => $m3[2]);	
		}
		$this->wikiConfig = $stor;
		
	}
	
	public function canWikiUseModule($siteName, $moduleName){
		if($this->wikiConfig == null){
			$this->loadWikiConfig();	
		}

		$row = $this->wikiConfig[$moduleName];
		
		if($row == null){
			return false;	
		}
		if($row['permissions'] == null){
			return true;	
		}
		
		$sites = explode(",", $row['permissions']);
		if(in_array($siteName, $sites)){
			return true;
		} else {
			return false;
		}
	}
	
}
