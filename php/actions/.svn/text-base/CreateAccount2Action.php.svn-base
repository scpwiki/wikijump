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

class CreateAccount2Action extends SmartyAction {

	protected static $EVCODE_SEED = 'someseed';

	public function perform($runData) {
	}
	
	public function acceptRulesEvent($runData){
		$accept = $runData->getParameterList()->getParameterValue("acceptrules");
		if(!$accept){
			throw new ProcessException(_("You must accept Terms of Service before proceeding."), "must_accept");	
		}

	}

	public function step0Event($runData) {

		// do it manually. change of rules.
		$pl = $runData->getParameterList();
		$name = ($pl->getParameterValue("name"));
		$email = ($pl->getParameterValue("email"));
		$password = ($pl->getParameterValue("password"));
		$password2 = ($pl->getParameterValue("password2"));
		
		$captcha = trim($pl->getParameterValue("captcha"));

		// validate now.
		
		$errors = array();
		
		//name
		$unixified = WDStringUtils::toUnixName($name);
		if(strlen($name)<2){
			$errors['name'] = _("You really should provide the screen name you want to use.");
		}elseif(strlen8($name)>20){
			$errors['name'] = _("Your screen name should not be longer than 20 characters.");
		}elseif(preg_match('/^[ _a-zA-Z0-9-\!#\$%\^\*\(\)]+$/', $name) == 0){
			$errors['name'] = _("Only alphanumeric characters (+a few special) can be used in the screen name.");	
		}elseif(strlen($unixified)<2){
			$errors['name'] = _("It seems there are too less alphanumeric characters in your screen name");	
		}else{
			
			//handle forbidden names
			$unixName = WDStringUtils::toUnixName($name);	
			
			$forbiddenUnixNames = explode("\n", file_get_contents(WIKIDOT_ROOT.'/conf/forbidden_user_names.conf'));
			foreach($forbiddenUnixNames as $f){
				if(preg_match($f, $unixName) >0){
					$errors['name']	= _('For some reason this name is not allowed or is reserved for future use.');	
				}	
			}
			
			// check if user does not exist
			$c = new Criteria();
			$c->add("unix_name", $unixified);
			$u = DB_OzoneUserPeer::instance()->selectOne($c);
			if($u != null){
				$errors['name'] = _("A user with this screen name (or very similar) already exists.");
			}	
		}
		
		// now check email
		if(strlen($email)<5){
			$errors['email'] = _("Please provide a valid email address.");	
		}elseif(strlen($email)>50){
			$errors['email'] = _("Please provide a valid email address - this one seems is to long.");		
		}elseif(preg_match("/^[_a-zA-Z0-9-]+(\.[_a-zA-Z0-9-]+)*@[a-zA-Z0-9-]+(\.[a-zA-Z0-9-]+)+$/", $email) ==0){
			$errors['email'] = _("Please provide a valid email address.");
		}else{
			// check if email is unique
			$c = new Criteria();
			$c->add("lower(email)", strtolower($email));
			$u = DB_OzoneUserPeer::instance()->selectOne($c);	
			if($u != null){
				$errors['email'] = _("A user with this email already exists.");
			}
		}
		
		// check password
		if(strlen8($password)<6){
			$errors['password'] = _("Please provide a password min. 6 characters long.");	
		}elseif(strlen8($password)>20){
			$errors['password'] = _("Password should not be longer than 20 characters.");		
		}elseif($password2 != $password){
			$errors['password2'] = _("Passwords are not identical.");
		}	
		
		// check language
		$lang = $pl->getParameterValue("language");
		if($lang !== "pl" && $lang !== "en"){
			$errors['language'] = _("Please select your preferred language.");	
		}
		
		// captcha
		$captcha = str_replace('0','O', $captcha);
		$captcha = strtoupper($captcha);
		if($captcha != strtoupper($runData->sessionGet("captchaCode"))){
			$errors['captcha'] = _("Human verification code is not valid.");	
		}
		
		if(!$pl->getParameterValue("tos")){
			$errors['tos'] = _("Please read and agree to the Terms of Service.");	
		}
		
		if(count($errors)>0){
			$runData->ajaxResponseAdd("formErrors", $errors);	
			throw new ProcessException("Form errors", "form_errors");
		}
			
		// store data in the session
		
		$data = array(
			'name' => $name,
			'email' => $email,
			'password' => $password	,
			'language' =>$lang
		);
		
		$runData->sessionAdd("ca_data", $data);

		// send email HERE:

		$data = $runData->sessionGet("ca_data");
		$email = $data['email'];
		$name = $data['name'];
		
		//generate the email verification code 
		
		$evcode = $runData->sessionGet('evcode');
		if(!$evcode){
			srand((double)microtime()*1000000);
			$string = md5(rand(0,9999));
			$evcode = substr($string, 2, 9);
		}
		
		//send a confirmation email to the user.
		$oe = new OzoneEmail();
		$oe->addAddress($email);
		$oe->setSubject(sprintf(_("%s - email verification"), GlobalProperties::$SERVICE_NAME));
		$oe->contextAdd('name', $name);
		$oe->contextAdd('email', $email);
		$oe->contextAdd('evcode', $evcode);
		$oe->contextAdd('sessionHash', md5($runData->getSession()->getSessionId() . self::$EVCODE_SEED));
		
		$oe->setBodyTemplate('RegistrationEmailVerification');
	
		if (!$oe->Send()) {
			throw new ProcessException(_("The email can not be sent to this address."), "email_failed");
		} 		
		$runData->sessionAdd('evcode', $evcode);
		
	}

	public function sendEmailVerEvent($runData){
		
		$data = $runData->sessionGet("ca_data");
		$email = $data['email'];
		$name = $data['name'];
		
		//generate the email verification code 
		
		$evcode = $runData->sessionGet('evcode');
		if($evcode == null){
			srand((double)microtime()*1000000);
			$string = md5(rand(0,9999));
			$evcode = substr($string, 2, 6);
		}
		
		//send a confirmation email to the user.
		$oe = new OzoneEmail();
		$oe->addAddress($email);
		$oe->setSubject(sprintf(_("%s- email verification"), GlobalProperties::$SERVICE_NAME));
		$oe->contextAdd('name', $name);
		$oe->contextAdd('email', $email);
		$oe->contextAdd('evcode', $evcode);
		
		$oe->setBodyTemplate('RegistrationEmailVerification');
	
		if (!$oe->Send()) {
			throw new ProcessException(_("The email can not be sent to this address."), "email_failed");
		} 		
		$runData->sessionAdd('evcode', $evcode);
	}
	
	public function finalizeEvent($runData, $skipEvcode = false){
		// get the form data
		$pl = $runData->getParameterList();
		
		if(!$skipEvcode){
			$evcode = $pl->getParameterValue("evcode", "AMODULE");
			
			//check if the email vercode is correct
			$evcode2 = $runData->sessionGet('evcode');
			if ($evcode !== $evcode2) {
				throw new ProcessException(_("Invalid email verification code."), "invalid_code");
			}
		}
		
		$data = $runData->sessionGet("ca_data");
		
		$name = $data['name'];
		$email = $data['email'];
		$password = $data['password'];
		$lang = $data['language'];
		
		$db = Database::connection();
		$db->begin();
		
		// check again if email and nick are not duplicate!
		
		$c = new Criteria();
		$c->add("lower(email)", strtolower($email));
		$u = DB_OzoneUserPeer::instance()->selectOne($c);	
		if($u != null){
			$runData->resetSession();
			throw new ProcessException(_("A user with this email already exists. Must have been created meanwhile... " .
					"Unfortunately you have to repeat the whole procedure. :-("), "user_exists");
		}
		
		$unixified = WDStringUtils::toUnixName($name);
		$c = new Criteria();
		$c->add("unix_name", $unixified);
		$u = DB_OzoneUserPeer::instance()->selectOne($c);
		if($u != null){
			$runData->resetSession();
			throw new ProcessException(_("A user with this name (or very similar) already exists. Must have been created meanwhile... " .
					"Unfortunately you have to repeat the whole procedure. :-("), "user_exists");
		}

		// add new user!!!

		$nuser = new DB_OzoneUser();
		/* email as the username!!! */
		$nuser->setName($email);
		$nuser->setEmail($email);
		$nuser->setPassword(md5($password));		
		
		$nuser->setNickName($name);
		$nuser->setUnixName($unixified);
		
		$nuser->setLanguage($lang);
		
		$date = new ODate();
		$nuser->setRegisteredDate($date);
		$nuser->setLastLogin($date);
		
		$nuser->save();

		// profile
		
		$profile = new DB_Profile();
		$profile->setUserId($nuser->getUserId());
		$profile->save();
		
		$us = new DB_UserSettings();
		$us->setUserId($nuser->getUserId());
		$us->save();
		
		// profile page
		
		$c = new Criteria();
		$c->add("unix_name", "template-en");
		$tsite = DB_SitePeer::instance()->selectOne($c);
		
		$c = new Criteria();
		$c->add("unix_name", "profiles");
		$nsite = DB_SitePeer::instance()->selectOne($c);
		$ncategory = DB_CategoryPeer::instance()->selectByName('profile', $nsite->getSiteId());

		$dup = new Duplicator;
		$dup->setOwner($nuser);
		
		$dup->duplicatePage(DB_PagePeer::instance()->selectByName($tsite->getSiteId(), 'profile:template'),
					$nsite,  $ncategory, 'profile:'.$nuser->getUnixName());
		
		$page = DB_PagePeer::instance()->selectByName($nsite->getSiteId(), 'profile:'.$nuser->getUnixName());
		
		$ou = new Outdater();
		$ou->pageEvent('new_page', $page);
		
		$db->commit();
		
		/* Handle originalUrl. */
		$originalUrl = $runData->sessionGet('loginOriginalUrl');
		if($originalUrl){
			$runData->ajaxResponseAdd('originalUrl', $originalUrl);
			if($runData->sessionGet('loginOriginalUrlForce')){
				$runData->ajaxResponseAdd('originalUrlForce', true);
			}
		}
		// reset session etc.
		$runData->resetSession();
		$runData->getSession()->setUserId($nuser->getUserId());
		setcookie("welcome", $nuser->getUserId(), time() + 10000000, "/", GlobalProperties::$SESSION_COOKIE_DOMAIN);
		setcookie(GlobalProperties::$SESSION_COOKIE_NAME_IE, $runData->getSessionId(), null, "/");
		
	}
	
	public function cancelEvent($runData){
		// reset session etc.
		$runData->resetSession();
	}

}
