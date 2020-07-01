<?php

/**
*
* Parses for implied line breaks indicated by newlines.
*
* @category Text
*
* @package Text_Wiki
*
* @author Paul M. Jones <pmjones@php.net>
* @author Michal Frackowiak
*
* @license LGPL
*
* @version $Id$
*
*/

/**
*
* Parses for implied line breaks indicated by newlines.
*
* This class implements a Text_Wiki_Parse to mark implied line breaks in the
* source text, usually a single carriage return in the middle of a paragraph
* or block-quoted text.
*
* @category Text
*
* @package Text_Wiki
*
* @author Paul M. Jones <pmjones@php.net>
* @author Michal Frackowiak
*
*/

class Text_Wiki_Parse_Newline extends Text_Wiki_Parse {

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
                        '([^\n])' .   # Match any character that is not a newline
                        '\n' .        # Then a newline
                        '(?!\n)' .    # ...so long as it is not followed by another newline
                        '/mx';

    /**
    *
    * Generates a replacement token for the matched text.
    *
    * @access public
    *
    * @param array &$matches The array of matches from parse().
    *
    * @return string A delimited token to be used as a placeholder in
    * the source text.
    *
    */

    function process(&$matches)
    {
        return $matches[1] .
            $this->wiki->addToken($this->rule)
           ;// . $matches[2];
    }
}
