<?php
/**
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
 * Creates a date string.
 *
 * @category Text
 *
 * @package Text_Wiki
 *
 * @author Michal Frackowiak
 *
 */
class Text_Wiki_Parse_Date extends Text_Wiki_Parse {



    public $regex =     '/' . 
                        '\[\[date\s+' . 
                        '([0-9]+)' .      # A number, for a given time. Required.
                        '(\s+.*?)?' .     # Optional extra parameters (format)
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

    	$options = array();
    	$options['timestamp'] = $matches[1];

    	$attr = $this->getAttrs(trim($matches[2]));
    	foreach($attr as $key => $a){
    		$options[$key] = $attr[$key];
    	}

    	$token = $this->wiki->addToken(
            $this->rule, $options
        );

        return $token;
    }

}
