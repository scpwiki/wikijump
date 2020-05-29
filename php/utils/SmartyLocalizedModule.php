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

abstract class SmartyLocalizedModule extends SmartyModule {
	
	public function render($runData){
	
		$uu = $runData->getUser();
		if($uu){
			$lang = $uu->getLanguage();
		
			switch($lang){
				case 'pl':
					$glang="pl_PL";
					$wp = "pl";
					break;
				case 'en':
					$glang="en_US";
					$wp = "www";
					break;
			}

			putenv("LANG=$glang"); 
			putenv("LANGUAGE=$glang"); 
			setlocale(LC_ALL, $glang.'.UTF-8');
		}
		
		$out = parent::render($runData);
		
		if($uu){
			$lang = $GLOBALS['lang'];
				
			switch($lang){
				case 'pl':
					$glang="pl_PL";
					break;
				case 'en':
					$glang="en_US";
					break;
			}

			putenv("LANG=$glang"); 
			putenv("LANGUAGE=$glang"); 
			setlocale(LC_ALL, $glang.'.UTF-8');
		}
		
		return $out;
	}

}
