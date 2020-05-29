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

if(!extension_loaded('xdiff')){
	@dl( 'xdiff.so' );
}

/**
 * A wrapper around xdiff.
 */
class ODiff {
	private $contextLines = 1;
	private $minimal = true;
	
	private $errors = null;
	
	public function setContextLines($val){
		$this->contextLines = $val;	
	}
	
	public function setMinimal($val){
		$this->minimal = $val;
	}	
	
	public function getErrors(){
		return $this->errors;	
	}
	
	public function diffString($stringFrom, $stringTo){
		// fix "no new lineat the end" problem.	
		if (!ereg("\n$",$stringFrom)) $stringFrom.="\n";
		if (!ereg("\n$",$stringTo)) $stringTo.="\n";
		return xdiff_string_diff($stringFrom, $stringTo);
	}
	
	public function patchString($string, $patch, $reverse = false){
		if (!ereg("\n$",$string)) $string.="\n";
		if (!ereg("\n$",$patch)) $patch.="\n";
		if($reverse == false){
			return xdiff_string_patch($string, $patch);
		}else{
			return xdiff_string_patch($string, $patch,XDIFF_PATCH_REVERSE);
		}
	}
	
}
