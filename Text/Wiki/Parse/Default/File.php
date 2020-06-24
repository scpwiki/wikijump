<?php
/**
*
* @category Text
*
* @package Text_Wiki
*
* @author Michal Frackowiak
*
* @license LGPL
*
* @version $Id$
*
*/

/**
*
* Parses attachement file links.
*
* @category Text
*
* @package Text_Wiki
*
* @author Michal Frackowiak
*
*/

class Text_Wiki_Parse_File extends Text_Wiki_Parse {

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
                        '\[\[file' . 
                        '\s+' . 
                        '(.+?)' .          # Name of file
                        '(?:\|(.+?))?' .   # Pipe, then link text (optional)
                        '\]\]' . 
                        '/ix';

    function process(&$matches)
    {

    		$file = trim($matches[1]);
    		$anchor = trim($matches[2]);

      	if($anchor == null || $anchor === ''){
      		$anchor = $file;
      	}

      	$options = array('file' => $file,
      					 'anchor' => $anchor);
        return $this->wiki->addToken($this->rule, $options);
    }
}
