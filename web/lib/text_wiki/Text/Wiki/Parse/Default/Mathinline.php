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
* Parses for text marked as inline math.
*
* This class implements a Text_Wiki_Parse to find math invocation code,
* i.e. [[$ equation $]]
*
* @category Text
*
* @package Text_Wiki
*
* @author Michal Frackowiak
*
*/

class Text_Wiki_Parse_Mathinline extends Text_Wiki_Parse {

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
                        '\[\[' . 
                        '\$' .      # $
                        '(.*?)' .   # Contents
                        '\$' .      # $
                        '\]\]' . 
                        '/x';
    /**
    *
    * Generates a token entry for the matched text.  Token options are:
    *
    * 'text' => The full matched text, not including the <code></code> tags.
    *
    * @access public
    *
    * @param array &$matches The array of matches from parse().
    *
    * @return A delimited token number to be used as a placeholder in
    * the source text.
    *
    */

    function process(&$matches)
    {
    	$content = trim($matches[1]);

        $options = array('content'=>$content);

        return $this->wiki->addToken($this->rule, $options);
    }
}
