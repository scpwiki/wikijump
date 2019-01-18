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

class NewPageHelperModule extends SmartyModule {
	
	public function build($runData){
		
		$site = $runData->getTemp("site");
		
		$pl = $runData->getParameterList();
		$categoryName = trim($pl->getParameterValue("category", "MODULE"));
		
		$template=trim($pl->getParameterValue("template", "MODULE"));
		
		$format=trim($pl->getParameterValue("format", "MODULE"));
		
		$runData->contextAdd("categoryName", WDStringUtils::toUnixName($categoryName));	
		
		if($template){
			$ta = explode(',', $template);
			$tp = array();
			foreach($ta as $t){
			// 	for each of the suggested arrays
				$t = trim($t);
				if(!preg_match("/^template:/",$t)){
					throw new ProcessException(sprintf(_('"%s" is not in the "template:" category.'), $t), "not_template");	
				}
				$page = DB_PagePeer::instance()->selectByName($site->getSiteId(), $t);	
				if($page == null){
					throw new ProcessException(sprintf(_('Template "%s" can not be found.'),$t), "no_template");
				}
				$tp[] = $page;
			}

			if(count($tp)>1){
				$runData->contextAdd("templates", $tp);	
			}
			if(count($tp) == 1){
				$runData->contextAdd("template", $tp[0]);	
			}
		}
		
		// size of the field
		
		$fieldSize = $pl->getParameterValue("size", "MODULE");
		$style = $pl->getParameterValue("style", "MODULE");
		$buttonText = $pl->getParameterValue("button", "MODULE");
		
		if(!$fieldSize){
			$fieldSize = 30;
		}
		
		$runData->contextAdd('size', $fieldSize);
		$runData->contextAdd('style', $style);
		$runData->contextAdd('button', $buttonText);
		
		// check if format is valid (vali regexp)
		$m = false;
		if($format){
			$m = @preg_match($format, 'abc');
		
			if($m !== false){
				$runData->contextAdd('format', $format);
			}else{
				$runData->contextAdd("formatError", $format);
			}
		}
		
	}
	
}
