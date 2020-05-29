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

class SimpleToDoModule extends SmartyModule {
    
    public static $_counter = 0 ;
    public static $_canEdit = false;
    public static $_labelArray = array();
    
    public function build($runData) {
        $user = $runData->getUser();
        if (self::$_counter == 0){
            // check permissions
            $page = $runData->getTemp("page");
            if($page){
                $category = $page->getCategory();//s$runData->getTemp("category");
			    try{
			        WDPermissionManager::instance()->hasPagePermission('create', $user, $category);
			        self::$_canEdit = true;
			    }catch(Exception $e){}
			 
			}
        }
        
        $runData->contextAdd('canEdit', self::$_canEdit);
        
        $runData->contextAdd('listCounter',self::$_counter) ;
        
        self::$_counter++ ;
        
        $pl = $runData->getParameterList();
        $label = $pl->getParameterValue("id");
        
        $label = trim($label);
        if(!$label){
            throw new ProcessException(_('The SimpleTodo module must have an id.'));
        }
        
        if (!in_array($label,self::$_labelArray)){
            array_push(self::$_labelArray,$label);
        } else {
            throw new ProcessException(_('The id attribute sholud be unique.'));
        }
        
        $runData->contextAdd("label", $label);
        $site = $runData->getTemp("site");
        $c = new Criteria();
        $c->add('label',$label);
        $c->add('site_id', $site->getSiteId());
        $list = DB_SimpletodoListPeer::instance()->selectOne($c);
        if ($list){
            $json = new JSONService();
            $listData = $json->decode($list->getData());
            $runData->contextAdd("title",$list->getTitle());
            $runData->contextAdd("data",$listData);
        }
        
    }  
}
