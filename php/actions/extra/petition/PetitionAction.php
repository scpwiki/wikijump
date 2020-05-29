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

class PetitionAction extends SmartyAction {
	
	public function perform($r){}
	
	public function signEvent($runData){

		require(WIKIDOT_ROOT.'/php/unclassified/country_codes.php');
		
		$site = $runData->getTemp("site");
		$pl = $runData->getParameterList();
		$campaignId = $pl->getParameterValue("campaignId");

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
		
		if(!$camp->getActive()){
			throw new ProcessException(_("This petition campaign is paused."));	
		}
		
		$errors = array();
		
		// prepare the new signature at the same time
		
		$pet = new DB_PetitionSignature();
		
		// first and last name
		$firstName = trim($pl->getParameterValue("firstName"));
		if(strlen($firstName) == 0){
			$errors['firstName'] = _("Please enter your first name.");
		}elseif(strlen8($firstName) >64){
			$errors['firstName'] = _("First name seems to be too long.");	
		}
		$lastName = trim($pl->getParameterValue("lastName"));
		if(strlen($lastName) == 0){
			$errors['lastName'] = _("Please enter your last name.");
		}elseif(strlen8($lastName) >64){
			$errors['lastName'] = _("Last name seems to be too long.");	
		}
		$pet->setFirstName($firstName);
		$pet->setLastName($lastName);
		
		// address
		if($camp->getCollectAddress()){
			$address1 =  trim($pl->getParameterValue("address1"));
			$address2 =  trim($pl->getParameterValue("address2"));
			if(strlen($address1) == 0){
				$errors['address'] = _("Please enter your address.");
			}elseif(strlen8($address1) >100){
				$errors['address'] = _("The address seems to be too long.");	
			}
			if(strlen8($address2) >100){
				$errors['address'] = _("The address seems to be too long.");
			}
			$pet->setAddress1($address1);
			$pet->setAddress2($address2);
		}
		
		//city
		if($camp->getCollectCity()){
			$city =  trim($pl->getParameterValue("city"));
			if(strlen($city) == 0){
				$errors['city'] = _("Please enter the city of residence.");
			}elseif(strlen8($city) >64){
				$errors['city'] = _("The city name seems to be too long.");	
			}
			$pet->setCity($city);
		}
		
		//state
		if($camp->getCollectState()){
			$state =  trim($pl->getParameterValue("state"));
			//}else
			if(strlen8($state) >64){
				$errors['state'] = _("The name of the state seems to be too long.");	
			}
			$pet->setState($state);	
		}
		
		//zip
		if($camp->getCollectZip()){
			$zip =  trim($pl->getParameterValue("zip"));
			if(strlen($zip) == 0){
				$errors['zip'] = _("Please enter your zip/postal code.");
			}elseif(strlen8($zip) >20){
				$errors['zip'] = _("The zip/postal code seems to be too long.");	
			}
			$pet->setZip($zip);
		}
		
		//country
		
		if($camp->getCollectCountry()){
			$country =  trim($pl->getParameterValue("country"));
			
			if(strlen($country) == 0 || !isset($iso3166_country_codes[$country])){
				$errors['country'] = _("Please choose your country.");
			}
			$pet->setCountryCode($country);
			$pet->setCountry($iso3166_country_codes[$country]);
			
			/*
			if(strlen($country) == 0){
				$errors['country'] = _("Please enter your country.");
			}elseif(strlen8($country) > 60){
				$errors['country'] = _("The name of the country is too long.");	
			}
			$pet->setCountry($country);
			*/
		}
		
		//comments
		if($camp->getCollectComments()){
			$comments =  trim($pl->getParameterValue("comments"));
			if(strlen8($comments) > 300){
				$errors['comments'] = _("The comments should not be longer than 300 characters.");
			}
			$pet->setComments($comments);
		}
		
		//verify email
		$email =  trim($pl->getParameterValue("email"));
		if(!preg_match('/^[_a-zA-Z0-9\-\+]+(\.[_a-zA-Z0-9\-\+]+)*@[a-zA-Z0-9-]+(\.[a-zA-Z0-9-]+)+$/', $email)){
			$errors['email'] = _("Please provide a valid email address.");
		}
		
		// check if email is unique for this campaign!
		if(!$errors['email']){
			$c = new Criteria();
			$c->add("campaign_id", $camp->getCampaignId());
			$c->add("email", $email);
			$pet0 = DB_PetitionSignaturePeer::instance()->selectOne($c);
			if($pet0){
				if($pet0->getConfirmed()){
					$errors['email'] = _("This email has been already used for signing the petition.");
				}else{
					DB_PetitionSignaturePeer::instance()->deleteByPrimaryKey($pet0->getSignatureId());	
				}
			}
		}
		
		$pet->setEmail($email);

		if(count($errors)>0){
			// there are some errors!!!
			$runData->ajaxResponseAdd("errors", $errors);
			throw new ProcessException(_("The form contains some errors."), "form_errors");	
			
		}
		
		// everything should be ok at this point - finish creating the signature, 
		// save the signature and send a verification email.
		
		$pet->setCampaignId($camp->getCampaignId());
		$pet->setDate(new ODate());
		
		// generate hash.
		
		$hash = substr(md5($email.time()),0,20);
		$pageUnixName = $pl->getParameterValue("petitionUrl");
		$pageUnixName = WDStringUtils::toUnixName($pageUnixName);
		$url = $site->getDomain().'/'.$pageUnixName;
		
		$pet->setConfirmationUrl($url);
		
		$oe = new OzoneEmail();
		$oe->addAddress($email);
		$oe->setSubject(_("Petition confirmation"));
		$oe->contextAdd('firstName', $firstName);
		$oe->contextAdd('lastName', $lastName);
		$oe->contextAdd('hash', $hash);
		$oe->contextAdd("site", $site);
		$oe->contextAdd("siteName", $site->getName());
		$oe->contextAdd("url", $url);
		$oe->contextAdd("campaign", $camp);
		$oe->contextAdd("campaignName", $camp->getName());
		$oe->contextAdd("sig", $pet);

		$oe->setBodyTemplate('wiki/petition/PetitionConfirmation');
		
		if(!$oe->Send()){
			throw new ProcessException(_("Confirmation email can not be delivered to the specified address."));
		}
		
		$pet->setConfirmationHash($hash);
		$pet->setConfirmationUrl('/'.$pageUnixName);
		$pet->save();

		$db->commit();
		
		$runData->setModuleTemplate("extra/petition/ConfirmationSentModule");
		
		$runData->sessionAdd("keep", true);
	}
	
	public function confirmEvent($runData){
		$site = $runData->getTemp("site");
		$pl = $runData->getParameterList();
		$campaignId = $pl->getParameterValue("campaignId");
		$hash = $pl->getParameterValue("hash");
		
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
		
		if(!$camp->getActive()){
			throw new ProcessException(_("This petition campaign is paused."));	
		}
		
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
		
		$pet->setConfirmed(true);
		$pet->setConfirmationUrl(null);
		
		$pet->save();
		
		$camp->updateNumberSignatures();
		$camp->save();
		
		// standard "thank you" or redirect?
		
		$thankYouPage = $camp->getThankYouPage();
		if($thankYouPage){
			$runData->ajaxResponseAdd("thankYouPage", $thankYouPage);
		}else{
			$runData->setModuleTemplate("extra/petition/SignatureConfirmedModule");
		}
		
		$db->commit();
	}
	
	public function cancelEvent($runData){
		$site = $runData->getTemp("site");
		$pl = $runData->getParameterList();
		$campaignId = $pl->getParameterValue("campaignId");
		$hash = $pl->getParameterValue("hash");
		
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
		
		$c = new Criteria();	
		$c->add("campaign_id", $camp->getCampaignId());
		$c->add("confirmation_hash", $hash);
		$c->add("confirmed", false);
		$pet = DB_PetitionSignaturePeer::instance()->selectOne($c);
		
		if(!$pet){
			throw new ProcessException(_("The petition signature can not be found."));	
		}
		
		DB_PetitionSignaturePeer::instance()->deleteByPrimaryKey($pet->getSignatureId());
		
		$runData->setModuleTemplate("extra/petition/SignatureCancelledModule");
		
		$db->commit();
	}
	
}
