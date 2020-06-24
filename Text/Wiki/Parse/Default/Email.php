<?php

/**
*
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
* Parses emails.
*
* @category Text
*
* @package Text_Wiki
*
* @author Michal Frackowiak
*
*/

class Text_Wiki_Parse_Email extends Text_Wiki_Parse {

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

    public $regex =     '[A-z0-9\-_]+' .           # At least one alphanumeric, underscore, hyphen
                        '(?:\.[_a-z0-9\-]+)*' .    # Allow a dot so long as it is not the first
                        '@[a-z0-9\-]+' .           # Characters after the @ symbol
                        '(?:\.[a-z0-9\-]+)+';      # Allow a dot so long as is is not the first
                        # TODO: Replace this trash, probably with a filter_var statement.

    function parse(){

    	 	// described emails
        $tmp_regex = '/\[(' . $this->regex . ') (.+?)\]/ix';
        $this->wiki->source = preg_replace_callback(
            $tmp_regex,
            array(&$this, 'processDescr'),
            $this->wiki->source
        );

    		// standalone emails
        $tmp_regex = '/' . $this->regex . '/ix';
        $this->wiki->source = preg_replace_callback(
            $tmp_regex,
            array(&$this, 'process'),
            $this->wiki->source
        );

    }

    function process(&$matches){
        $options = array(
            'email' => $matches[0],
            'text' => $matches[0]
        );

        return $this->wiki->addToken($this->rule, $options);
    }

	function processDescr(&$matches){
        $options = array(
           'email' => $matches[1],
            'text' => $matches[2]
        );

        return $this->wiki->addToken($this->rule, $options);
    }

}
