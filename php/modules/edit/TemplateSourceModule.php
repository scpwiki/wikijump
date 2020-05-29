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

class TemplateSourceModule extends SmartyModule{
	
	public function build($runData){
		$pageId = $runData->getParameterList()->getParameterValue("page_id");
		$page = DB_PagePeer::instance()->selectByPrimaryKey($pageId);
		$source = $page->getSource();
//		/* Determine if it is a live template. */
//		if(preg_match(';%%content({[0-9]+})?%%;', $source)) {
//		    $split = array();
//	        $split = preg_split(';^=default={4,}$;sm', $source);
//	        if(count($split) == 2){
//	            /* Fine, there is some initial content. */
//	            $source = trim($split[1]);   
//	        } else {
//	            $source = null;
//	        }
//		}
		$runData->contextAdd("source", $source);
	}
	
}
