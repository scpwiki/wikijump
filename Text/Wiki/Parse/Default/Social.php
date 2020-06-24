<?php

/**
*
*
* @category Text
*
* @package Text_Wiki
*
* @author Paul M. Jones <pmjones@php.net>
*
* @license LGPL
*
* @version $Id$
*
*/

/**
*
* Parses for [[social]] tag.
*
* @category Text
*
* @package Text_Wiki
*
* @author Michal Frackowiak
*
*/

class Text_Wiki_Parse_Social extends Text_Wiki_Parse {

    /**
    *
    * The regular expression used to find source text matching this
    * rule.
    *
    * @access public
    *
    * @var string
    *
    */

    public $regex =     '/' . 
                        '\[\[social' . 
                        '(\s+[^\]]+?)?' .   # Parameters
                        '\]\]' . 
                        '/isx';

    function process(&$matches)
    {
        $sites = trim($matches[1]);

       	$d = utf8_encode("\xFE");

       	$out = $d."module \"wiki/social/SocialBookmarksModule\"";
	    	if($sites!==null && $sites!=='') {$out.=" ".urlencode('sites="'.$sites.'"')." ";}
	    	$out.=$d;

	    	return $out;

       /* $sites = trim($matches[1]);
	   	$options = array('sites' => $sites);

        return $this->wiki->addToken($this->rule, $options);
        */
    }
}
