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
 * Object Model class.
 *
 */
class DB_Theme extends DB_ThemeBase {

	protected $_external;
	
    public function getStyleUrls() {
        
        if ($this->getExtendsThemeId()) {
            // get parent theme
            $parent = DB_ThemePeer::instance()->selectByPrimaryKey($this->getExtendsThemeId());
            if ($parent == null) {
                throw new ProcessException("FATAL ERROR: Theme not found.");
            }
            $files = $parent->getStyleUrls();
        } else {
            $files = array();
        }
        
        $files[] = $this->getStyleUrl();
        return $files;
    }

    /**
     * Returns url of the style associated with this theme.
     */
    public function getStyleUrl() {
        if ($this->getCustom()) {
            return "/local--theme/" . $this->getUnixName() . "/style.css?" . $this->getRevisionNumber();
        } elseif($this->_external){
        	return $this->_external;
        } else {
            return "/common--theme/" . $this->getUnixName() . "/css/style.css?" . $this->getRevisionNumber();
        }
    }

    public function getThemePreview() {
        return DB_ThemePreviewPeer::instance()->selectByPrimaryKey($this->getThemeId());
    }
    
    public function setExternalUrl($url){
    	$this->_external = $url;
    }

}
