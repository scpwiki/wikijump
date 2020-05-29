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

class SimpleToDoAction extends SmartyAction {
    
    public function perform($r){
        
    }

    public $dataArray = array();
    
    public function saveEvent($runData){
        $site = $runData->getTemp("site");
        $pl = $runData->getParameterList();
        $pageId = $pl->getParameterValue("pageId");
        if (!is_numeric($pageId)){
            throw new ProcessException(_("Page does not exist."));
        }
		$page = DB_PagePeer::instance()->selectByPrimaryKey($pageId);

		if(!$page) {
			throw new ProcessException(_("Page does not exist."));
		}
			
			// check permissions
		$category = $page->getCategory();
		WDPermissionManager::instance()->hasPagePermission('edit', $runData->getUser(), $category, $page);

        $data = $pl->getParameterValue("data");
        $json = new JSONService();
        $listData = $json->decode($data);
        //it's time to do some checking
        $listData->label = trim($listData->label);
        if(!$listData->label){
            throw new ProcessException(_('The SimpleTodo module must have an id (e.g. id="list1").'));
        }
        $dataArray['label'] = $listData->label;
        $listData->title = trim($listData->title);
        if(!$listData->title){
            throw new ProcessException(_('Your title field is empty, please correct that.'));
        }
        $dataArray['title'] = $listData->title ;
        for($i=0; $i<count($listData->data); $i++){
            $listData->data[$i]->text = trim($listData->data[$i]->text) ;
            $listData->data[$i]->link = trim($listData->data[$i]->link) ;
            if (!is_bool($listData->data[$i]->checked)){
                throw new ProcessException(_('Something is wrong witch checkbox (it is not a boolean value).'));
            }
            if (empty($listData->data[$i]->text)){
                throw new ProcessException(_('One of your text fields is empty, please correct that.'));
            }
            $dataArray['data'][$i]['text'] = $listData->data[$i]->text;
            $dataArray['data'][$i]['link'] = $listData->data[$i]->link ;
            $dataArray['data'][$i]['checked'] = $listData->data[$i]->checked ;
        }
        
        $c = new Criteria();
        $c->add('label',$listData->label);
        $c->add('site_id', $site->getSiteId());
        $list = DB_SimpletodoListPeer::instance()->selectOne($c);
        
        if (!$list){
            $list = new DB_SimpletodoList();
            $list->setSiteId($site->getSiteId());
            $list->setLabel($dataArray['label']);
        }
        $list->setTitle($dataArray['title']);
        $itemData = $json->encode($dataArray['data']);
        $list->setData($itemData);
        $list->save();
    }
}
