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

class ManageSiteEditCustomThemeModule extends ManageSiteBaseModule {
	
	public function build($runData){
		
		$pl = $runData->getParameterList();
		
		$site = $runData->getTemp("site");
		$runData->contextAdd("site", $site);

		// now select themes that can be extended

		$c = new Criteria();
	
		$c->add("custom", false);
	
		$c->addOrderAscending("sort_index");
		$c->addOrderAscending("name");
		$themes = DB_ThemePeer::instance()->select($c);
		$runData->contextAdd("exthemes", $themes);
		
		$themeId = $pl->getParameterValue("themeId");
		if($themeId && is_numeric($themeId)){
			$theme = DB_ThemePeer::instance()->selectByPrimaryKey($themeId);
			if($theme== null || $theme->getSiteId() !== $site->getSiteId()){
				throw new ProcessException(_("Error selecting theme."), "wrong_theme");	
			}
			$runData->contextAdd("theme", $theme);
			$dir = WIKIDOT_ROOT."/web/files--sites/".$site->getUnixName()."/theme/".$theme->getUnixName();
			$code = file_get_contents($dir."/style.css");
			$runData->contextAdd("code", $code);
		}

	}
	
}
