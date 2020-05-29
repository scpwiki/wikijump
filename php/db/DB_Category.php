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
 * @package Wikidot_Db
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

/**
 * Object Model mapped class.
 *
 */
class DB_Category extends DB_CategoryBase {

	public function getLicenseText(){
		if($this->getName() === '_default'){
			if($this->getLicenseId() == 1){
				return $this->getLicenseOther();
			} else {
				$license = DB_LicensePeer::instance()->selectById($this->getLicenseId());
				return $license->getDescription();	
			}	
		} else {
			if($this->getLicenseDefault()){
				// get default license (for the '_default' category
				$dc = DB_CategoryPeer::instance()->selectByName('_default', $this->getSiteId());
				return $dc->getLicenseText();		
			} else {
				if($this->getLicenseId() == 1){
					return $this->getLicenseOther();
				} else {
					$license = DB_LicensePeer::instance()->selectById($this->getLicenseId());
					return $license->getDescription();
				}		
			}
		}
		
	}
	
	public function getTopPage(){
		if($this->getName() === '_default'){
			$pageName = $this->getTopBarPageName();
		} else {
			if($this->getNavDefault()){
				// get default category
				$dc = DB_CategoryPeer::instance()->selectByName('_default', $this->getSiteId());
				$pageName = $dc->getTopBarPageName();
			} else {
				$pageName = $this->getTopBarPageName();
			}	
		}
		// now GET this page ;-)
		$page = DB_PagePeer::instance()->selectByName($this->getSiteId(), $pageName);
		return $page;
	}	
	
	public function getSidePage(){
		if($this->getName() === '_default'){
			$pageName = $this->getSideBarPageName();
		} else {
			if($this->getNavDefault()){
				// get default category
				$dc = DB_CategoryPeer::instance()->selectByName('_default', $this->getSiteId());
				$pageName = $dc->getSideBarPageName();
			} else {
				$pageName = $this->getSideBarPageName();
			}	
		}
		// now GET this page ;-)
		$page = DB_PagePeer::instance()->selectByName($this->getSiteId(), $pageName);
		return $page;
	}	
	
	public function getTheme(){
		if($this->getExternalTheme()){
			$theme = $this->getExternalTheme();
			if($this->getName() !== '_default'){
				if($this->getThemeDefault()){
					$dc = DB_CategoryPeer::instance()->selectByName('_default', $this->getSiteId());
					$theme = $dc->getTheme();
				} else {
					$theme = DB_ThemePeer::instance()->selectByPrimaryKey($this->getThemeId());
				}
			}
			return $theme;
		}
		
		
		if($this->getName() === '_default'){
			$theme = DB_ThemePeer::instance()->selectByPrimaryKey($this->getThemeId());
		} else {
			if($this->getThemeDefault()){
				$dc = DB_CategoryPeer::instance()->selectByName('_default', $this->getSiteId());
				$theme = $dc->getTheme();
			} else {
				$theme = DB_ThemePeer::instance()->selectByPrimaryKey($this->getThemeId());
			}
		}
		return $theme;
	}
	
	protected function getExternalTheme(){
		if(!$this->getThemeExternalUrl()){
			return null;
		}
		$t = new DB_Theme();
		$t->setExternalUrl($this->getThemeExternalUrl());
		/* Get base theme. */
		$c = new Criteria();
		$c->add('name', 'Base');
		$c->add('custom', false);
		$baseTheme = DB_ThemePeer::instance()->selectOne($c);
		$t->setExtendsThemeId($baseTheme->getThemeId());
		$t->setThemeId($baseTheme->getThemeId()); // needed sometime
		return $t;
	}
	
	public function getPermissionString(){
		if($this->getName() === '_default' || !$this->getPermissionsDefault()){
			$ps = $this->getPermissions();
		} else {
			$dc = DB_CategoryPeer::instance()->selectByName('_default', $this->getSiteId());
			$ps = $dc->getPermissions();
		}
		return $ps;
	}
	
	public function getShowDiscuss(){
		$ppd = $this->getPerPageDiscussion();
		if($ppd === null && $this->getName() !== '_default'){
			$dc = DB_CategoryPeer::instance()->selectByName('_default', $this->getSiteId());
			$ppd = $dc->getPerPageDiscussion();	
		}
		if($ppd === null){
			$ppd = false;
		}	
		return $ppd;	
	}
	
	public function getRatingString(){
		$ppd = $this->getRating();
		if(strpos($ppd, 'e') === false && strpos($ppd, 'd') === false && $this->getName() !== '_default'){
			$dc = DB_CategoryPeer::instance()->selectByName('_default', $this->getSiteId());
			$ppd = $dc->getRating();	
		}
		if($ppd === null){
			$ppd = 'd';
		}	
		return $ppd;		
	}
	
	public function getRatingEnabled(){
		$s = $this->getRating();
		if(strpos($s, 'e') !== false){
			return true;
		}elseif(strpos($s, 'd') !== false){
			return false;
		}else{
			return null;	
		}
	}
	public function getRatingEnabledEff(){
		$s = $this->getRatingString();
		if(strpos($s, 'e') !== false){
			return true;
		}elseif(strpos($s, 'd') !== false){
			return false;
		}else{
			return null;	
		}
	}

	public function getRatingType(){
		$s = $this->getRatingString();
		preg_match('/(P|M|S)/', $s, $m);
		$m = $m[0];
		if(!$m) {$m = 'P';}
		return $m;
	}
	
	public function getRatingBy(){
		$s = $this->getRatingString();
		if(strpos($s, 'm') !== false){
			return 'm';
		}else{
			return 'r';
		}
	}
	
	public function getRatingVisible(){
		$s = $this->getRatingString();
		if(strpos($s, 'v')!== false){
			return 'v';
		}else{
			return 'a';
		}
	}
	
	public function save(){
		$memcache = Ozone::$memcache;
		$key = 'category..'.$this->getSiteId().'..'.$this->getName();
		$memcache->delete($key);
		$key = 'categorybyid..'.$this->getSiteId().'..'.$this->getCategoryId();
		$memcache->delete($key);
		
		if($this->getPerPageDiscussion() === null && $this->getName() == '_default'){
			$this->setPerPageDiscussion(false);
		}
		parent::save();	
	}
	
}
