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

class SignPetitionModule extends SmartyModule {
	
	public function build($runData){
		
		require(WIKIDOT_ROOT.'/php/unclassified/country_codes.php');
		$runData->contextAdd("coutryCodes", $iso3166_country_codes);

		$site = $runData->getTemp("site");
		$pl = $runData->getParameterList();
		$id = $pl->getParameterValue("id");
		
		if(!$id){
			throw new ProcessException(_("The campaign identifier is not valid."));		
		}

		$c = new Criteria();
		$c->add("site_id", $site->getSiteId());
		$c->add("deleted", false);
		$c->add("identifier", $id);
		
		$camp = DB_PetitionCampaignPeer::instance()->selectOne($c);
		
		if(!$camp){
			throw new ProcessException(_("The campaign can not be found."));	
		}
		
		if(!$camp->getActive()){
			throw new ProcessException(_("This petition campaign is paused."));	
		}
		
		$runData->contextAdd("campaign", $camp);
		
		$confirm = $pl->getParameterValue("confirm");
		
		if($confirm){
//			// working in the CONFIRMATION mode!
//			
//			// get the petition
//			
//			

			$db = Database::connection();
			$db->begin();

			// get the petition
			$hash = $confirm;
			$c = new Criteria();	
			$c->add("campaign_id", $camp->getCampaignId());
			$c->add("confirmation_hash", $hash);
			$pet = DB_PetitionSignaturePeer::instance()->selectOne($c);
			
			if(!$pet){
				throw new ProcessException(_("The petition signature can not be found."));	
			}
			if($pet->getConfirmed()){
				throw new ProcessException(_("This signature has been already confirmed."));	
			}
			
			// confirm it and redirect to a "thank you" page or display "thank you".
			
			$pet->setConfirmed(true);
			$pet->setConfirmationUrl(null);
			
			$pet->save();
			
			$camp->updateNumberSignatures();
			$camp->save();
			
			$db->commit();
			
			$thankYouPage = $camp->getThankYouPage();
			if($thankYouPage){
				// simply REDIRECT!
				header("HTTP/1.1 301 Moved Permanently");
				header("Location: /".$thankYouPage);
				exit();
			}else{
				$runData->setModuleTemplate("extra/petition/SignatureConfirmedModule");
			}
			
		}

	}
	
}
