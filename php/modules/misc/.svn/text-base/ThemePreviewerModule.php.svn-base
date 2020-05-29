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

class ThemePreviewerModule extends SmartyModule {
	
	protected $processPage = true;
	
	private $themeId;
	private $theme;
	
	public function build($runData){
		
		$site = $runData->getTemp("site");
		
		$pl = $runData->getParameterList();

		$themeId = $pl->getParameterValue('theme_id');
		if($themeId === null){
			$themeUrl = current($_GET); // BAD HACK!!!
			if($themeUrl){
				$theme = $this->getExternalTheme($themeUrl);
			}else{
			
				$page = $runData->getTemp("page");
				if($page == null){
					throw new ProcessException(_("Not working in the preview mode. Not a preview mode? So it might be an error."));	
				}
				$theme = $page->getCategory()->getTheme();
			}
		}else{
			$theme = DB_ThemePeer::instance()->selectByPrimaryKey($themeId);
		}
		
		//$this->themeId = $themeId;
		
		if($theme == null || $theme->getAbstract() == true || 
			($theme->getCustom ==true && $theme->getSiteId() != $site->getSiteId())){
			
			throw new ProcessException(_("Error selecting theme."));
					
		} 
		
		$this->theme = $theme;
		
		$q = "SELECT * from theme WHERE " .
				"abstract = FALSE  " .
				"AND (custom = FALSE" .
					" OR (custom = TRUE AND site_id='".$site->getSiteId()."' AND site_id !=1)" .
				") " .
				"ORDER BY custom, sort_index, replace(name, '_', '00000000');";
			
		$c = new Criteria();		
		$c->setExplicitQuery($q);
		$themes = DB_ThemePeer::instance()->select($c);
		$runData->contextAdd("themes", $themes);
		
		$runData->contextAdd("currentTheme", $theme);
		$runData->contextAdd("noUi", $pl->getParameterValue('noUi'));
			
	}
	
	public function processPage($out, $runData){
		
		$theme = $this->theme;
		$t = '';
		foreach($theme->getStyleUrls() as $url){
   			$t .= "@import url($url);\n";
		}
		
		$out = preg_replace('/(@import url\([^\)]*?style\.css(\?[0-9]+)?\);\s*)+/s', $t, $out, 1);
			
		return $out;
	}
	
	protected function getExternalTheme($url){
		if(!$url){
			return null;
		}
		$t = new DB_Theme();
		$t->setExternalUrl($url);
		/* Get base theme. */
		$c = new Criteria();
		$c->add('name', 'Base');
		$c->add('custom', false);
		$baseTheme = DB_ThemePeer::instance()->selectOne($c);
		$t->setExtendsThemeId($baseTheme->getThemeId());
		$t->setThemeId($baseTheme->getThemeId()); // needed sometime
		return $t;
	}

}
