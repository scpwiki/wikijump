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

class RateAction extends SmartyAction{
	
	private $message;
	
	public function perform($r){}
	
	public function ratePageEvent($runData){
		$pl = $runData->getParameterList();
		$site = $runData->getTemp("site");
		
		$pageId = $pl->getParameterValue("pageId");
		$user = $runData->getUser();
		
		$points = $pl->getParameterValue("points");
		
		if($points != 1 && $points != -1){
			throw new ProcessException("Error");	
		}
		
		$page = DB_PagePeer::instance()->selectByPrimaryKey($pageId);
		if(!$pageId || $page == null || $page->getSiteId() != $runData->getTemp("site")->getSiteId()){
			throw new ProcessException(_("Error getting page information."), "no_page");
		}	
		
		// check if allowed
		if(!$this->canRatePage($user, $page)){
			// prepare the message
			
			throw new ProcessException($this->message);	
		}
		
		// ok, now check if not rated already...

		$c = new Criteria();
		$c->add("page_id", $page->getPageId());
		$c->add("user_id", $user->getUserId());
		if($pl->getParameterValue("force")){
			$v = DB_PageRateVotePeer::instance()->selectOne($c);
			DB_PageRateVotePeer::instance()->delete($c);
			$rpoints = $points - $v->getRate();	
		}else{
			$v = DB_PageRateVotePeer::instance()->selectOne($c);
			if($v){
				$runData->ajaxResponseAdd("status", "already_voted");
				$runData->setModuleTemplate("pagerate/AlreadyRatedModule");
				$runData->contextAdd("points", $points);
				$runData->contextAdd("rate", $v->getRate());
				return;
				//throw new ProcessException("It seems you have already voted here. " .
			}
			$rpoints = $points;	
		}
		
		//ok, now VOTEEEEeeeeee!!!!!
		
		$db = Database::connection();
		$db->begin();
		
		$v = new DB_PageRateVote();
		$v->setUserId($user->getUserId());
		$v->setPageId($page->getPageId());
		$v->setRate($points);
		$v->setDate(new ODate());
		
		$v->save();
		
		// update page points
		$q = "UPDATE page SET rate=COALESCE((" .
				"SELECT sum(rate) FROM page_rate_vote WHERE page_id = '".$page->getPageId()."'),0) " .
				"WHERE page_id='".$page->getPageId()."'";
		
		$db->query($q);
		$outdater = new Outdater();
		$outdater->pageEvent("page_vote", $page);

		$db->commit();
			
		$runData->ajaxResponseAdd("points", $rpoints);
	}
	
	public function cancelVoteEvent($runData){
		$pl = $runData->getParameterList();
		$site = $runData->getTemp("site");
		
		$pageId = $pl->getParameterValue("pageId");
		$user = $runData->getUser();
		
		$page = DB_PagePeer::instance()->selectByPrimaryKey($pageId);
		if(!$pageId || $page == null || $page->getSiteId() != $runData->getTemp("site")->getSiteId()){
			throw new ProcessException(_("Error getting page information."), "no_page");
		}	
		
		// check if allowed
		if(!$this->canRatePage($user, $page)){
			// prepare the message
			
			throw new ProcessException($this->message);	
		}
		
		$db = Database::connection();
		$db->begin();
		
		$c = new Criteria();
		$c->add("page_id", $page->getPageId());
		$c->add("user_id", $user->getUserId());
		$v = DB_PageRateVotePeer::instance()->selectOne($c);
		if(!$v) {
			$runData->ajaxResponseAdd("points",0);
			return;
		}
		DB_PageRateVotePeer::instance()->delete($c);
		$rpoints = 0 - $v->getRate();	
		
		// update page points
		$q = "UPDATE page SET rate=COALESCE((" .
				"SELECT sum(rate) FROM page_rate_vote WHERE page_id = '".$page->getPageId()."'),0) " .
				"WHERE page_id='".$page->getPageId()."'";
		
		$db->query($q);
		$outdater = new Outdater();
		$outdater->pageEvent("page_vote", $page);
		
		$db->commit();	
		
		$runData->ajaxResponseAdd("points", $rpoints);
	}
	
	private function canRatePage($user, $page){
		if(!$user){
			$this->message = _("You should be at least logged in to try rating...");
			return false;
		}
		
		$category = $page->getCategory();
		if($category->getRatingEnabledEff()){
			$ps = $category->getRatingBy();
			if($ps == 'r'){
				return true;
			}
		
			if($ps == 'm'){
				$c = new Criteria();
				$c->add("site_id", $category->getSiteId());
				$c->add("user_id", $user->getUserId());
				$rel = DB_MemberPeer::instance()->selectOne($c);
				if($rel){
					return true;
				}else{
					$this->message = _("Voting is enabled only for Site Members.");
				}	
			}
		}else{
			$this->message = _("Rating is disabled for this page.");	
		}
		return false;	
	}
	
}
