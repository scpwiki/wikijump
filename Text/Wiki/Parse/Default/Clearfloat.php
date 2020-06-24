<?php

/**
 *
 * Clears floats.
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
 * Clears floats.
 *
 * @category Text
 *
 * @package Text_Wiki
 *
 * @author Michal Frackowiak
 *
 */

class Text_Wiki_Parse_Clearfloat extends Text_Wiki_Parse {

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
                        '([~]{4,})' .   # ~~~~
                        '(>|<)?' .      # Optional directional modifier
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

    function process(&$matches) {
        $side = $matches[2];
        $options = array('side' => $side);
        return "\n\n" . $this->wiki->addToken($this->rule, $options) . "\n\n";
    }
}
