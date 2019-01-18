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

function smarty_function_pager($params, & $smarty) {
	
	$lang = Ozone::$runData->getLanguage();
	if($lang == 'en'){
		$li18 = array(
			'page' => 'Page',
			'of' => 'of',
			'next' => 'next',
			'previous' => 'previous'
		);
	}elseif($lang == 'pl'){
		$li18 = array(
			'page' => 'Strona',
			'of' => 'z',
			'next' => 'następna',
			'previous' => 'poprzednia'
		);
	}
	
	$jsfunction = $params['jsfunction'];
	$url = $params['url'];
	
	$total = $params['total'];
	$known = $params['known'];
	$current = $params['current'];

	if($total){
		$pages = $total;	
	}
	if($known){
		$pages = $known;	
	}
	
	if($current == $known){
		$total = $known;
	}
	
	if(!($total>1 || $known>1)){
		return;
	}
	
	$a = array();
	
	if($current > 1){
		$a[] = array('&laquo; '._('previous'), $current-1);	
	}
	
	if($current > 3){
		$a[] = array('1', 1);
	}
	if($current > 4){
		$a[] = array('2', 2);
	}	
	if($current == 6){
		$a[] = array('3', 3);
	}
	if($current > 6){
		$a[] = array('...');
	}
	if($current-2 >= 1){
		$a[] = array($current-2, $current-2);
	}
	if($current-1 >= 1){
		$a[] = array($current-1, $current-1);
	}
	$a[] = array($current, $current, true); // CURRENT PAGE!
	if($current+1 <= $pages){
		$a[] = array($current+1, $current+1);
	}
	if($current+2 <= $pages){
		$a[] = array($current+2, $current+2);
	}
	if($current < $pages -5 ){
		$a[] = array('...');
	}
	if($current == $pages-5){
		$a[] = array($pages-2, $pages-2);
	}
	if($current < $pages-3){
		$a[] = array($pages-1, $pages-1);
	}
	if($known != null && $current != $known){
		$a[] = array('...');	
	}
	if($current < $pages-2){
		$a[] = array($pages, $pages);
	}
	
	if($current != $pages){
		$a[] = array(_('next').' &raquo;', $current+1);	
	}
	
	$out = "";
	$out .= '<div class="pager">';
	$out .= '<span class="pager-no">'._('page').' '.$current;
	if($total ){
		$out .= ' '._('of').' '.$total;
	}
	$out .= '</span>';	
	
	foreach($a as $p){
		if(isset($p[1])){
			if($p[2] == true){
				// current page
				$class = 'current';	
				$out .= '<span class="'.$class.'">'.$p[0].'</span>';
			}else {
				$class = 'target';
				if($jsfunction){
					if(strpos($jsfunction, '#')){
						$js = str_replace('#',$p[1],$jsfunction);	
					}else{
						$js = 	$jsfunction.'('.$p[1].')';
					}
					$out .= '<span class="'.$class.'"><a href="javascript:;" onclick="'.$js.'">'.$p[0].'</a></span>';	
				}elseif($url){
					$out .= '<span class="'.$class.'"><a href="'.sprintf(preg_replace('/%([A-E0-9]{2,2})/i', '%%\\1',$url), $p[1]).'">'.$p[0].'</a></span>';	
				}
			}
			
		}else{
			$out .= '<span class="dots">'.$p[0].'</span>';
		}
	}

	$out.="</div>";
	// prepare an array of page(numbers) to display

	return $out;
		
}
