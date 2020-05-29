<?php

/**
* 
* Parses for module delimiter characters already in the source text.
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
* Parses for module delimiter characters already in the source text.
*
* @category Text
* 
* @package Text_Wiki
* 
* @author Michal Frackowiak
* 
*/

class Text_Wiki_Parse_Moduledelimiter extends Text_Wiki_Parse {
    
    /**
    * 
    * Constructor.  Overrides the Text_Wiki_Parse constructor so that we
    * can set the $regex property dynamically (we need to include the
    * Text_Wiki $delim character.
    * 
    * @param object &$obj The calling "parent" Text_Wiki object.
    * 
    * @param string $name The token name to use for this rule.
    * 
    */
    
    function Text_Wiki_Parse_delimiter(&$obj)
    {
        parent::Text_Wiki_Parse($obj);
        $this->regex = "/".utf8_encode("\xFE")."/";
    }

    /**
    * 
    * Generates a token entry for the matched text.  Token options are:
    * 
    * 'text' => The full matched text.
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
        return $this->wiki->addToken(
            $this->rule,
            array('text' => utf8_encode("\xFE"))
        );
    }
}
