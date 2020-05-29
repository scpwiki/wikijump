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

class LoginStatusModule extends Module{

	public function render($runData){
		// use non-templating solution to have some optimisation here. not to call
		// Smarty if not required!
		
		$user = $runData->getUser();
		
		if($user == null){
			$site = $runData->getTemp('site');
			$originalUrl = 'http://' . $_SERVER['HTTP_HOST'] . $_SERVER['REQUEST_URI'];
			if(preg_match(';\?origUrl=.*$;', $originalUrl)){
				$o = array();
				parse_str(preg_replace(';^.*?\?;', '', $_SERVER['REQUEST_URI']), $o);
				$originalUrl = $o['origUrl'];
			}
			$loginDomain = 'www';
			if($site->getLanguage() != 'en'){
				$loginDomain = $site->getLanguage();
			}
			$out = '<a href="http://'.$loginDomain.'.'.GlobalProperties::$URL_DOMAIN . '/auth:newaccount?origUrl='.urlencode($originalUrl).'">'._('create account').'</a> '._('or') . ' ';
			$out .= '<a href="http://'.$loginDomain.'.'.GlobalProperties::$URL_DOMAIN . '/auth:login?origUrl='.urlencode($originalUrl).'">'._('login').'</a> ';
			
			//$out = '<a href="javascript:;" onclick="WIKIDOT.page.listeners.createAccount(event)">'._('create account').'</a> '._('or').' <a href="javascript:;" onclick="WIKIDOT.page.listeners.loginClick(event)">'._('login').'</a>';
		} else {
			
			$lang = $user->getLanguage();
		
			switch($lang){
				case 'pl':
					$glang="pl_PL";
					break;
				case 'en':
					$glang="en_US";
					break;
			}

			putenv("LANG=$glang"); 
			putenv("LANGUAGE=$glang"); 
			setlocale(LC_ALL, $glang.'.UTF-8');
		
			$userId = $user->getUserId();
			$linkInner = 'href="http://' . GlobalProperties::$URL_HOST . '/user:info/'.$user->getUnixName().'" onclick="WIKIDOT.page.listeners.userInfo('.$user->getUserId().'); return false;" ';
			
			$out = '<span class="printuser"><a '.$linkInner.'><img class="small" src="/common--images/avatars/'.floor($userId/1000).'/'.$userId.'/a16.png" alt="avatar"';
			/* karma: */
			$out .= ' style="background-image:url(http://' . GlobalProperties::$URL_HOST . '/userkarma.php?u=' .$userId  . ')"';
			/* end of karma */
			$out .= '/></a>';
			$out .= $user->getNickName().'</span>'.
					' | <a href="http://'.GlobalProperties::$URL_HOST.'/account:you">'._('my account').'</a>' .
					'<a  id="account-topbutton" href="javascript:;">&nabla;</a>';
			$out .= '<div id="account-options">' .
					'<ul>' .
					'<li><a href="http://'.GlobalProperties::$URL_HOST.'/account:you">'._('account summary').'</a></li>' .
					'<li><a href="http://'.GlobalProperties::$URL_HOST.'/account:you/start/messages">'._('private messages').'</a></li>' .
					'<li><a href="http://'.GlobalProperties::$URL_HOST.'/account:you/start/contacts">'._('my contacts').'</a></li>' .
					'<li><a href="http://'.GlobalProperties::$URL_HOST.'/account:you/start/notifications">'._('notifications').'</a></li>'.
					'<li><a href="http://'.GlobalProperties::$URL_HOST.'/account:you/start/watched-changes">'._('watched pages').'</a></li>'.
					'<li><a href="http://'.GlobalProperties::$URL_HOST.'/account:you/start/watched-forum">'._('watched discussions').'</a></li>'.
					'<li><a href="javascript:;" onclick="WIKIDOT.page.listeners.logoutClick(event)">'._('logout').'</a></li>' .
					'</ul></div>';

            // back the language!

            $lang = $GLOBALS['lang'];
            
            switch ($lang) {
                case 'pl':
                    $glang = "pl_PL";
                    break;
                case 'en':
                    $glang = "en_US";
                    break;
            } 

            putenv("LANG=$glang");
            putenv("LANGUAGE=$glang");
            setlocale(LC_ALL, $glang . '.UTF-8');
        }

        return $out; 	
	}	
}
