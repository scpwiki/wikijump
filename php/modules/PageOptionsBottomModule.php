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

class PageOptionsBottomModule extends Module {
	
	public function render($runData){
		
		// quickly check - show or not.
		
		$pl = $runData->getParameterList();
		$pageName = $runData->getTemp("pageUnixName");
		
		$page = $runData->getTemp("page");//$pl->getParameterValue("page", "MODULE");
		
		// get category name and get the category by name.
		// this should be enchanced to use memcache later
		// to get category to avoid db connection.
		
		// extract category name
		if(strpos($pageName, ':') != false){
			// ok, there is category!	
			$exp = explode(':',$pageName);
			$categoryName = $exp[0];
		} else {
			$categoryName = "_default";	
		}
		$site = $runData->getTemp("site");
		$category = DB_CategoryPeer::instance()->selectByName($categoryName, $site->getSiteId());
		$user = $runData->getUser();
		
		$pm = new WDPermissionManager();
		$pm->setThrowExceptions(false);
		$pm->setCheckIpBlocks(false); // to avoid database connection.
		if(!$pm->hasPagePermission('options', $user, $category, $pageName, $site)){
			return '';	
		}
		
		$showDiscuss = $pl->getParameterValue("showDiscuss");
		if($showDiscuss){
			$threadId = $pl->getParameterValue("threadId");
			$pageUnixName = $pl->getParameterValue("pageUnixName");
		}
		
		$showRate = $category->getRatingEnabledEff();

		// now a nasty part - make it inline such that
		// the Smarty engine does need to be initialized.
		// nasty way but saves a LOT of time with caching enabled.
		
		$otext = '';
		
		if($page){
		
			$otext .= 	'<div id="page-info">'.
					_('page_revision').': '.$page->getRevisionNumber().', '.
					_('last_edited').': <span class="odate">'.
					$page->getDateLastEdited()->getTimestamp().
					'|%e %b %Y, %H:%M %Z (%O '._('ago').')</span>'.
					'</div>';

		}

		$otext .= '
<div id="page-options-bottom"  class="page-options-bottom">
	<a href="javascript:;" id="edit-button">'._('edit').'</a>';
	
	if($showRate&&$page){
		$otext .= 	'<a href="javascript:;" id="pagerate-button">'._('rate').' (<span id="prw54355">'.($page->getRate()>0?'+':''). $page->getRate().'</span>)</a>';
	}
	
	$otext .= '<a href="javascript:;" id="tags-button">'._('tags').'</a>';

		if($showDiscuss&&$page){
			if($threadId){
				$no = $page->getTemp("numberPosts");
				$otext.='<a href="/forum/t-'.$threadId.'/'.$pageUnixName.'"  id="discuss-button">'._('discuss').' ('.$no.')</a>';	
			}else{
				$otext.='<a href="javascript:;" id="discuss-button" onclick="WIKIDOT.page.listeners.createPageDiscussion(event)">'._('discuss').'</a> ';
			}
				
		}
		
		$otext .= '
	<a href="javascript:;" id="history-button">'._('history').'</a> 
	<a href="javascript:;" id="files-button">'._('files').'</a> ' .
	'<a href="javascript:;" id="print-button">'._('print').'</a> ' .
	'<a href="javascript:;" id="site-tools-button">'._('site tools').'</a>';
	if($site->getPrivate()){
	}
	$otext .= '<a href="javascript:;" id="more-options-button">+&nbsp;'._('options').'</a> 
</div>
<div id="page-options-bottom-2" class="page-options-bottom" style="display:none">
	<a href="javascript:;" id="edit-sections-button">'._('edit sections').'</a>
	<a href="javascript:;" id="edit-append-button">'._('append').'</a>
	<a href="javascript:;" id="backlinks-button">'._('backlinks').'</a> 
	<a href="javascript:;" id="view-source-button">'._('view source').'</a> 
	<a href="javascript:;" id="parent-page-button">'._('parent').'</a> 
	<a href="javascript:;" id="page-block-button">'._('block').'</a> 	
	<a href="javascript:;" id="rename-move-button">'._('rename').'</a> 
	<a href="javascript:;" id="delete-button">'._('delete').'</a> 
</div>
<div id="page-options-area-bottom">
</div>
';

		return $otext;
			
	}

}
