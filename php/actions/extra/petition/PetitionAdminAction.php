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

class PetitionAdminAction extends SmartyAction {
	
	public function isAllowed($runData){
		WDPermissionManager::instance()->hasPermission('manage_site', $runData->getUser(), $runData->getTemp("site"));	
		return true;
	}
	
	public function perform($r){}
	
	public function createCampaignEvent($runData){
		
		$site = $runData->getTemp("site");
		$pl = $runData->getParameterList();
		
		$name =$pl->getParameterValue("name");
		$id = $pl->getParameterValue("identifier"); 
		
		//validate data

		$name = trim($name);
		$id = trim($id);
		if($name == ""){
			throw new ProcessException(_("You must provide a name for this campaign."), "form_error");
		}
		if($id == ""){
			throw new ProcessException(_("You must provide an identifier for this campaign."), "form_error");
		}
		if(!preg_match(';^[a-z0-9]+$;i',$id)){
			throw new ProcessException(_("The identifier can contain only letters and digits."), "form_error");
		}
		if(strlen8($name)>50){
			throw new ProcessException(_("Name of the campaign shoud not be longer than 50 characters."), "form_error");	
		}
		if(strlen8(id)>20){
			throw new ProcessException(_("Identifier of the campaign shoud not be longer than 20 characters."), "form_error");	
		}
		
		$db = Database::connection();
		$db->begin();

		// check if camaign exists already!
		
		$c = new Criteria();
		$c->add("site_id", $site->getSiteId());
		$c->add("deleted", false);
		$c->add("name", $name);
		$camp0 = DB_PetitionCampaignPeer::instance()->selectOne($c);
		if($camp0){
			throw new ProcessException(_("A campaign with this name already exists."), "form_error");	
		}
		
		$c = new Criteria();
		$c->add("site_id", $site->getSiteId());
		$c->add("deleted", false);
		$c->add("identifier", $id);
		$camp0 = DB_PetitionCampaignPeer::instance()->selectOne($c);
		if($camp0){
			throw new ProcessException(_("A campaign with the same identifier already exists."), "form_error");	
		}
		
		// ok, this seems to be all right!
		
		$camp = new DB_PetitionCampaign();
		$camp->setName($name);
		$camp->setIdentifier($id);
		$camp->setSiteId($site->getSiteId());
		
		$camp->save();
		$runData->ajaxResponseAdd("campaignId", $camp->getCampaignId());
		
		$db->commit();
		
	}
	
	public function suspendCampaignEvent($runData){
		$site = $runData->getTemp("site");
		$pl = $runData->getParameterList();
		$campaignId = $pl->getParameterValue("campaignId");
		
		$c = new Criteria();
		$c->add("site_id", $site->getSiteId());
		$c->add("deleted", false);
		$c->add("campaign_id", $campaignId);
		
		$camp = DB_PetitionCampaignPeer::instance()->selectOne($c);
		
		if(!$camp){
			throw new ProcessException(_("The campaign can not be found."));	
		}
		
		$camp->setActive(false);
		$camp->save();
	}
	
	public function resumeCampaignEvent($runData){
		$site = $runData->getTemp("site");
		$pl = $runData->getParameterList();
		$campaignId = $pl->getParameterValue("campaignId");
		
		$c = new Criteria();
		$c->add("site_id", $site->getSiteId());
		$c->add("deleted", false);
		$c->add("campaign_id", $campaignId);
		
		$camp = DB_PetitionCampaignPeer::instance()->selectOne($c);
		
		if(!$camp){
			throw new ProcessException(_("The campaign can not be found."));	
		}
		
		$camp->setActive(true);
		$camp->save();
	}
	
	public function deleteCampaignEvent($runData){
		$site = $runData->getTemp("site");
		$pl = $runData->getParameterList();
		$campaignId = $pl->getParameterValue("campaignId");
		
		$c = new Criteria();
		$c->add("site_id", $site->getSiteId());
		$c->add("deleted", false);
		$c->add("campaign_id", $campaignId);
		
		$camp = DB_PetitionCampaignPeer::instance()->selectOne($c);
		
		if(!$camp){
			throw new ProcessException(_("The campaign can not be found."));	
		}
		
		$camp->setDeleted(true);
		$camp->setIdentifier('deleted:'.$camp->getIdentifier());
		$camp->save();
	}
	
	public function saveCollectEvent($runData){
		$site = $runData->getTemp("site");
		$pl = $runData->getParameterList();
		$campaignId = $pl->getParameterValue("campaignId");
		$thankYouPage = WDStringUtils::toUnixName($pl->getParameterValue("thankYouPage")); 
		
		$c = new Criteria();
		$c->add("site_id", $site->getSiteId());
		$c->add("deleted", false);
		$c->add("campaign_id", $campaignId);
		
		$camp = DB_PetitionCampaignPeer::instance()->selectOne($c);
		
		if(!$camp){
			throw new ProcessException(_("The campaign can not be found."));	
		}
		
		// so, what to collect and show?
		$co = array();
		$sh = array();
		$co['address'] = (bool) $pl->getParameterValue("collectAddress");
		$co['city'] = (bool) $pl->getParameterValue("collectCity");
		$sh['city'] = (bool) $pl->getParameterValue("showCity");
		
		$co['state'] = (bool) $pl->getParameterValue("collectState");
		$sh['state'] = (bool) $pl->getParameterValue("showState");
		
		$co['zip'] = (bool) $pl->getParameterValue("collectZip");
		$sh['zip'] = (bool) $pl->getParameterValue("showZip");
		
		$co['country'] = (bool) $pl->getParameterValue("collectCountry");
		$sh['country'] = (bool) $pl->getParameterValue("showCountry");
		
		$co['comments'] = (bool) $pl->getParameterValue("collectComments");
		$sh['comments'] = (bool) $pl->getParameterValue("showComments");
		
		// check if the landing page exists
		if($thankYouPage){
			$page = DB_PagePeer::instance()->selectByName($site->getSiteId(), $thankYouPage);
			if(!$page){
				throw new ProcessException('The "thank you" page does not exist'); 	
			}
		}	
		
		$camp->setCollectAddress($co['address']);
		$camp->setCollectCity($co['city']);
		$camp->setShowCity($sh['city']);
		$camp->setCollectState($co['state']);
		$camp->setShowState($sh['state']);
		$camp->setCollectZip($co['zip']);
		$camp->setShowZip($sh['zip']);
		$camp->setCollectCountry($co['country']);
		$camp->setShowCountry($sh['country']);
		$camp->setCollectComments($co['comments']);
		$camp->setShowComments($sh['comments']);
		
		$camp->setThankYouPage($thankYouPage);

		$camp->save();
		
	}
	
	public function removeSignaturesEvent($runData){
		$site = $runData->getTemp("site");
		$pl = $runData->getParameterList();
		$campaignId = $pl->getParameterValue("campaignId");
		$ids = $pl->getParameterValue("ids");
		
		$db = Database::connection();
		$db->begin();
		
		$c = new Criteria();
		$c->add("site_id", $site->getSiteId());
		$c->add("deleted", false);
		$c->add("campaign_id", $campaignId);
		
		$camp = DB_PetitionCampaignPeer::instance()->selectOne($c);
		
		if(!$camp){
			throw new ProcessException(_("The campaign can not be found."));	
		}
		
		$ids = explode(',', $ids);
		// create a delete query.
		$c1 = new Criteria();
		foreach($ids as $id){
			$c1->addOr("signature_id", $id);	
		}
		
		$c = new Criteria();
		$c->add("campaign_id", $camp->getCampaignId());
		$c->addCriteriaAnd($c1);
		
		DB_PetitionSignaturePeer::instance()->delete($c);
		
		$camp->updateNumberSignatures();
		$camp->save();
		
		$db->commit();
	}
	
}
