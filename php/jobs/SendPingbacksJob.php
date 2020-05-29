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
 * @package Wikidot_Cron
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

/**
 * Sends pingbacks.
 */
class SendPingbacksJob implements SchedulerJob {

    public function run() {
        Database::init();
        
        while($link = $this->_selectLink()){
        	$this->_ping($link);
        }
    }
    
    protected function _selectLink() {
    	$date = new ODate();
        $date->subtractSeconds(3600);
        $date0 = new ODate();
        $date0->subtractSeconds(600);
        
        $q = "SELECT page_external_link.* FROM page_external_link, site_settings, site, category, page" .
        	 " WHERE page_external_link.pinged = false AND page_external_link.date > '".db_escape_string($date->getDate())."' " .
        	 " AND page_external_link.date < '".db_escape_string($date0->getDate())."' " .
             " AND (category.enable_pingback_out = true OR site_settings.enable_all_pingback_out = true) " .
        	 " AND site.private = false AND site.visible=true AND site.deleted=false " .
        	 " AND page_external_link.page_id = page.page_id" . 
        	 " AND page.category_id = category.category_id AND category.site_id = site_settings.site_id AND page.site_id = site.site_id LIMIT 1";
        $c = new Criteria();
        $c->setExplicitQuery($q);
        $link = DB_PageExternalLinkPeer::instance()->selectOne($c);
        return $link;
    }
    
    protected function _ping($link){
    	$link->setPinged(true);
    	$link->setPingStatus('PROCESSING');
    	$link->save();
    	
    	$h = $link->buildPageUrl();
    	
    	$ping = new PingBack($link->getToUrl(), $h);
    	try{
    		$status = $ping->ping();
    		$link->setPingStatus($status);
    	}catch(PingBackException $e){
    		$link->setPingStatus($e->getMessage());
    	}catch(Exception $e) {
    		$link->setPingStatus('EXCEPTION');
    	}
    	$link->save();
    	//echo $h;
    }
}
