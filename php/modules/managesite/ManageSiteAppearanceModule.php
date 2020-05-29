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

class ManageSiteAppearanceModule extends ManageSiteBaseModule {
	
	public function build($runData){
		
		$site = $runData->getTemp("site");
		$runData->contextAdd("site", $site);
		
		// get all categories for the site
		$c = new Criteria();
		$c->add("site_id", $site->getSiteId());
		$c->addOrderAscending("replace(name, '_', '00000000')");
		$categories = DB_CategoryPeer::instance()->select($c);
		
		$runData->contextAdd("categories", $categories);
		
		// also prepare categories to put into javascript...
		$cats2 = array();
		foreach($categories as $category){
			$arr = $category->getFieldValuesArray();
			// change themes to conform to variants structure
			if($arr['theme_id']){
				$theme = DB_ThemePeer::instance()->selectByPrimaryKey($category->getThemeId());
				if($theme->getVariantOfThemeId() != null){
					$arr['theme_id'] = $theme->getVariantOfThemeId();
					$arr['variant_theme_id'] = $theme->getThemeId();
					$arr['theme_external_url']	= $category->getThemeExternalUrl();
				}
			}
			$cats2[] = $arr;
		}
		$runData->ajaxResponseAdd("categories", $cats2);
		
		// now select themes
		$c = new Criteria();
		/*$c->add("custom", false);
		$c->add("abstract", false);
		$c->addOrderAscending("name");*/
		$q = "SELECT * from theme WHERE " .
				"abstract = FALSE AND variant_of_theme_id IS NULL " .
				"AND (custom = FALSE" .
					" OR (custom = TRUE AND site_id='".$site->getSiteId()."')" .
				") " .
				"ORDER BY custom, sort_index, replace(name, '_', '00000000');";
					
		$c->setExplicitQuery($q);
		$themes = DB_ThemePeer::instance()->select($c);
		$runData->contextAdd("themes", $themes);
		
		// get theme variants too
		$c = new Criteria();
		$q = "SELECT * FROM theme WHERE variant_of_theme_id IS NOT NULL ORDER BY name";
		$c->setExplicitQuery($q);
		$variants =  DB_ThemePeer::instance()->select($c);
		
		$variantsArray = array();
		foreach($variants as $v){
			$variantsArray[$v->getVariantOfThemeId()][] = $v;	
		}
		
		$runData->contextAdd("variantsArray", $variantsArray);

	}
	
}
