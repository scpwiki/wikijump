<?php

/**
*
* Parses for horizontal ruling lines.
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
* Parses for content separator.
*
* This class implements a Text_Wiki_Parse to find source text marked to
* be a horizontal rule, as defined by four dashed on their own line.
*
* @category Text
*
* @package Text_Wiki
*
* @author Paul M. Jones <pmjones@php.net>
*
*/

class Text_Wiki_Parse_Separator extends Text_Wiki_Parse {

    /**
    *
    * The regular expression used to parse the source text and find
    * matches conforming to this rule.  Used by the parse() method.
    *
    * @access public
    *
    * @var string
    *
    * @see parse()
    *
    */

    public $regex =     '/' . 
                        '^' . 
                        '([=]{4,})' . 	# "====", four or more =
                        '$' . 
                        '/mx';

    /**
    *
    * Generates a replacement token for the matched text.
    *
    * @access public
    *
    * @param array &$matches The array of matches from parse().
    *
    * @return string A token marking the horizontal rule.
    *
    */

    function process(&$matches)
    {
        return "\n".$this->wiki->addToken($this->rule)."\n\n";
    }
}
