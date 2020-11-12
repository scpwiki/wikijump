<?php


/**
 * Language-variant block for Smarty.
 */

function smarty_block_ltext($params, $content, &$smarty, &$repeat)
{
    if (isset($content)) {
        $lang = $params['lang'];
        if($lang == ""){
        	//produce some error?
        	return;
        }
        if($lang == Ozone::$runData->getLanguage()){
        	return $content;
        } else {
        	return;
        }

    }
}
