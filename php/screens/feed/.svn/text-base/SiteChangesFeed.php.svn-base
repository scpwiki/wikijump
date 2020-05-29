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

class SiteChangesFeed extends FeedScreen {

	public function render($runData){
		$site = $runData->getTemp("site");
		$key = "sitechangesfeed..".$site->getSiteId();
		
		$mc = OZONE::$memcache;
		$out = $mc->get($key);
		if($out){
			return $out;	
		}
		$out = parent::render($runData);
		$mc->set($key, $out, 0, 3600);
		return $out;
	}
	
	public function build($runData){
	
		$site = $runData->getTemp("site");

		$c = new Criteria();
		
		$c->addJoin("page_id", "page.page_id");
		$c->addJoin("user_id", "ozone_user.user_id");
		$c->add("page.site_id", $site->getSiteId());
		$c->addOrderDescending("page_revision.revision_id");
		$c->setLimit(30);
		
		$revisions = DB_PageRevisionPeer::instance()->select($c);

		$channel['title'] = _('Recent page changes from site').' "'.htmlspecialchars($site->getName()).'" (a Wikidot site)';
		$channel['link'] = "http://".$site->getDomain();
		
		$items = array();
		
		foreach($revisions as $rev){
			$page = $rev->getPage();
			
			$item = array();

			$item['link'] = 'http://'.$site->getDomain().'/'.$page->getUnixName();
			
			$desc = '';
			
			$flags = array();
			if($rev->getFlagText()){
				$flags[] = _("source change");	
			}
			if($rev->getFlagTitle()){
				$flags[] = _("title change");
			}
			if($rev->getFlagFile()){
				$flags[] = _("file action");	
			}
			if($rev->getFlagRename()){
				$flags[] = _("page move/rename");	
			}
			if($rev->getFlagMeta()){
				$flags[] = _("metadata changed");
			}
			if($rev->getFlagNew()){
				$flags[] = _("new page");	
			}
			
			$item['title'] = '"'.$page->getTitleOrUnixName().'" - '.implode(', ', $flags);
			$desc = '';
			$desc .= _('Page').': <a href="http://'.$site->getDomain().'/'.$page->getUnixName().'">'.htmlspecialchars($page->getTitle()).'</a> ('.$page->getUnixName().')<br/>';
			$desc .= _('Current revision number').': '.$rev->getRevisionNumber().'<br/>';
			$desc .= _('Date changed').': '.date('r', $rev->getDateLastEdited()->getTimestamp()).'<br/>';
			$desc .= _('Change type').': '.implode(', ', $flags).'<br/>';
			if($rev->getComments()){
				$desc .= _('Change comments').': '.htmlspecialchars($rev->getComments()).'<br/>';
			}
			$desc .= _('By').': '.WDRenderUtils::renderUser($rev->getUserOrString()).'<br/>';
			
			$desc .= '<br/>'._('Page content preview').': <br/>'.$page->getPreview();
			$item['description'] = $desc;
			
			$item['content'] = $desc;
			
			$item['guid'] = $item['link']."#revision-".$rev->getRevisionId();
			$item['date'] = date('r', $rev->getDateLastEdited()->getTimestamp());
			
			$content =  '';

			$items[] = $item;	
		}
		
		$runData->contextAdd("channel", $channel);
		$runData->contextAdd("items", $items);
	}
	
}
