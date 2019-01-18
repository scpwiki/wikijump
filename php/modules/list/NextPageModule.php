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
 * @version $Id: ListPagesModule.php,v 1.10 2008/05/27 13:27:06 redbeard Exp $
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

require_once(WIKIDOT_ROOT . '/php/modules/list/ListPagesModule.php');

class NextPageModule extends ListPagesModule {
    
	protected $orderType = 'Asc';
	protected $listPagesParam = 'nextBy';
	
	/**
	 * 
	 * @param $runData RunData
	 */
    public function render($runData) {
    	$runData->setModuleTemplate("list/ListPagesModule");
    	
    	$pl = $runData->getParameterList();
    	$by = $pl->getParameterValue('by');
    	$pl->delParameter('by');
    	
    	if ($by == 'title') {
    		$by = 'title';
    		$order = "title" . $this->orderType;
    	} else {
    		$by = 'page_id';
    		$order = "dateCreated" . $this->orderType;
    	}
    	
    	$pl->addParameter($this->listPagesParam, $by, 'MODULE');
    	$pl->addParameter('order', $order, 'MODULE');
    	$pl->addParameter('limit', 1, 'MODULE');
    	
    	return parent::render($runData);
    }
}
