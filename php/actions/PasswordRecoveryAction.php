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

class PasswordRecoveryAction extends SmartyAction {
	
	public function perform($r){}
	
	public function step1Event($runData){
		$pl = $runData->getParameterList();
		
		$email = $pl->getParameterValue("email", "AMODULE");
		if($email == null || $email == ''){
			throw new ProcessException(_("Email must be provided."), "no_email");	
		}	
		
		$email = trim(CryptUtils::rsaDecrypt($email));
		$email = preg_replace("/^__/", '', $email);
		
		if($email == null || $email == ''){
			throw new ProcessException(_("Email must be provided."), "no_email");	
		}
		
		if(preg_match("/^[_a-zA-Z0-9-]+(\.[_a-zA-Z0-9-]+)*@[a-zA-Z0-9-]+(\.[a-zA-Z0-9-]+)+$/", $email) ==0){
			throw new ProcessException(_("Valid email must be provided."), "no_email");	
		}	
		
		// check for users with the email
		$c = new Criteria();
		$c->add("lower(email)", strtolower($email));
		$user = DB_OzoneUserPeer::instance()->selectOne($c);
		
		if($user == null){
			throw new ProcessException(_("This email can not be found in our database."), "no_email");		
		}
		
		// generate code
		srand((double)microtime()*1000000);
		$string = md5(rand(0,9999));
		$evcode = substr($string, 2, 6);

		//send a confirmation email to the user.
		$oe = new OzoneEmail();
		$oe->addAddress($email);
		$oe->setSubject(sprintf(_("%s - password recovery"), GlobalProperties::$SERVICE_NAME));
		$oe->contextAdd("user", $user);
		$oe->contextAdd("email", $email);
		$oe->contextAdd('revcode', $evcode);
		
		$oe->setBodyTemplate('PasswordRecoveryEmail');
	
		if (!$oe->Send()) {
			throw new ProcessException(_("The email can not be sent to this address."), "no_email");
		} 		 
		
		$runData->sessionAdd("revcode", $evcode);
		$runData->sessionAdd("prUserId", $user->getUserId());
		$runData->contextAdd("email", $email);
	}
	
	public function step2Event($runData){
		$pl = $runData->getParameterList();
		
		$evercode = $pl->getParameterValue("evercode");
		
		if($evercode != $runData->sessionGet("revcode")){
			throw new ProcessException(_("The verification codes do not match."), "form_error");
		}
		
		$password = $pl->getParameterValue("password");
		$password2 = $pl->getParameterValue("password2");
		
		$password = trim(CryptUtils::rsaDecrypt($password));
		$password = preg_replace("/^__/", '', $password);
		$password2 = trim(CryptUtils::rsaDecrypt($password2));
		$password2 = preg_replace("/^__/", '', $password2);
		
		// check password
		if(strlen8($password)<6){
			throw new ProcessException( _("Please provide a password min. 6 characters long."),"form_error");	
		}elseif(strlen8($password)>20){
				throw new ProcessException( _("Password should not be longer than 20 characters."),"form_error");		
		}elseif($password2 != $password){
				throw new ProcessException( _("Passwords are not identical."),"form_error");	
		}	
		
		// ok. seems fine.
		
		$userId = $runData->sessionGet("prUserId");
		$user = DB_OzoneUserPeer::instance()->selectByPrimaryKey($userId);
		if($user == null){
			throw ProcessException("No such user.", "no_user");	
		}
		
		$user->setPassword(md5($password));
		$user->save();
		
	}
	
	public function cancelEvent($runData){
		// reset session etc.
		$runData->resetSession();
	}
	
}
