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

class ManageSiteModule extends ManageSiteBaseModule {
	
	protected $processPage = true;
	
	public function build($runData){	
		
		$pl = $runData->getParameterList();
		$start = $pl->getParameterValue("start");
		if($start){
			$runData->contextAdd("start", $start);	
		}
		
		$site = $runData->getTemp("site");
		
		$runData->contextAdd("site", $site);
		
		$runData->contextAdd('useCustomDomains', GlobalProperties::$USE_CUSTOM_DOMAINS);
		$runData->contextAdd('useSsl', GlobalProperties::$USE_SSL);
	}
	
	public function processPage($out, $runData){
		
		$out = preg_replace("/<div id=\"page\-title\">(.*?)<\/div>/is","",$out, 1);
		
		return $out;	
	}
}
