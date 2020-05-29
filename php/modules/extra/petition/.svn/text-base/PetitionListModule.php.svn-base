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

class PetitionListModule extends SmartyModule{
	
	public function build($runData){
		$pl = $runData->getParameterList();
		$site = $runData->getTemp("site");
		// get the petition campaign...
		$campaignId = $pl->getParameterValue("id");
		
		$c = new Criteria();
		$c->add("site_id", $site->getSiteId());
		$c->add("deleted", false);
		$c->add("identifier", $campaignId);
		
		$camp = DB_PetitionCampaignPeer::instance()->selectOne($c);
		
		if(!$camp){
			throw new ProcessException(_("The campaign can not be found."));	
		}
		
		// get signatures!
		
		$limit = $pl->getParameterValue("limit");
		if($limit === null || !is_numeric($limit)){
			$limit = 50;
		}
		
		$c = new Criteria();
		$c->add("campaign_id", $camp->getCampaignId());
		$c->add("confirmed", true);
		$c->addOrderDescending("signature_id");
		if($limit > 0){
			$c->setLimit($limit);
		}
		$signatures = DB_PetitionSignaturePeer::instance()->select($c);
		
		$runData->contextAdd("signatures", $signatures);
		$runData->contextAdd("campaign", $camp);		
	}
	
}
