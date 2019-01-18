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

class MailFormUtils {
	
	public static function parseFormat($format){
		preg_match_all("/^#\s+([a-z0-9_\-]+)\s*:?((?:\n(?:\s+\*.*))+)/mi", $format, $matches, PREG_SET_ORDER);	

		$fields = array();
		foreach($matches as $f){
			$field = array();
			$field['name'] = $f[1];	
			
			$parameters = $f[2];
			
			// ok, should the parameters be parsed? at least some. or all.
			preg_match_all("/^ \*\s*([a-z0-9\-_]+)\s*:\s*(.*)$/mi", $parameters, $m2, PREG_SET_ORDER);
			foreach($m2 as $parameter){
				$field[$parameter[1]] = $parameter[2];	
			}
			
			// if "select", look for options
			if($field['type'] == "select"){
				preg_match_all("/^ \*\s+options\s*:?((?:\n(?:  \*.*))+)/mi", $parameters, $m3, PREG_SET_ORDER);		
				$optionsf = $m3[1];
			
				preg_match_all("/^  \*\s*([a-z0-9\-_]+)\s*:\s*(.*)$/mi", $parameters, $m4, PREG_SET_ORDER);
				$field['options'] = array();
				foreach($m4 as $option){
					$field['options'][$option[1]] = $option[2];
				}
				
				if(!in_array($field['default'], array_keys($field['options']))){
					unset($field['default']);	
				}	
			}

			// check if there are any rulezz^H^Hs
			preg_match_all("/^ \*\s+rules\s*:?((?:\n(?:  \*.*))+)/mi", $parameters, $m5, PREG_SET_ORDER);	
			if(count($m5)>0){
				preg_match_all("/^  \*\s*([a-z0-9]+)\s*:\s*(.*)$/mi", $parameters, $m5, PREG_SET_ORDER);
				$field['rules'] = array();
				foreach($m5 as $rule){
					$field['rules'][$rule[1]] = $rule[2];
				}
			}
			$fields[] = $field;
		}
		return $fields;	
	}
}
