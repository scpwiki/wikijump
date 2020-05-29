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

class WDRenderUtils {
	
	public static function renderUser($user, $params=array()){
		
		if($user == null || $user == ''){return null;}
		if(is_string($user)){
			$linkInner = 'href="javascript:;"  onclick="WIKIDOT.page.listeners.anonymousUserInfo(\''.$user.'\'); return false;" ';
			// 	ok, this is just "anonymous info". print it!
			$out = '<span class="printuser anonymous">';
			if($params['image'] != null){
				$image = $params['image'];
				// handle sizes...
				$out .= 	'<a '.$linkInner.' ><img class="small" src="/common--images/avatars/default/a16.png" alt=""/></a>';
			}
			$out .= '<a '.$linkInner.'>'._('Anonymous');
			list($ip, $proxy) = explode("|",$user);
			
			if(!$params['noip']) {$out .= ' <span class="ip">('.htmlspecialchars($ip).')</span>';}
			
			$out .= '</a></span>';	
			return $out;
		}
		
		$userId = $user->getUserId();
		
		if($userId<0){
			// always mean some kind of system bot. just print bot name.
			$out = '<span class="printuser">'.htmlspecialchars($user->getNickName()).'</span>';
			return $out;
		}
		
		$class = "printuser";
		if($params['image'] && $params['image'] !== 'small'){
			$class .= " avatarhover";	
		}
		
		$out = '<span class="'.$class.'">';
		$linkInner = 'href="http://' . GlobalProperties::$URL_HOST . '/user:info/'.$user->getUnixName().'" onclick="WIKIDOT.page.listeners.userInfo('.$user->getUserId().'); return false;" ';
		if($params['image'] != null){
			$image = $params['image'];
			// handle sizes...
			$out .= 	'<a '.$linkInner.' ><img class="small" src="/common--images/avatars/'.floor($userId/1000).'/'.$userId.'/a16.png" alt="'.htmlspecialchars($user->getNickName()).'"';
			/* karma: */
			$out .= ' style="background-image:url(http://' . GlobalProperties::$URL_HOST . '/userkarma.php?u=' . $userId . ')"';
			/* end of karma */
			$out .= '/></a>';
		}
		if(!$params['noNameLink']){
			$out .= '<a '.$linkInner.'>'.htmlspecialchars($user->getNickName()).'</a></span>';
		}else{
			htmlspecialchars($user->getNickName());
		}
		return $out;	
		
	}
	
}
