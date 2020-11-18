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
 * Joins lines.
 *
 * @category Text
 *
 * @package Text_Wiki
 *
 * @author Michal Frackowiak
 *
 */
class Text_Wiki_Parse_Concatlines extends Text_Wiki_Parse {

    /**
     *
     * Simple parsing method.
     *
     * @access public
     *
     */

    function parse() {

        // concat lines ending in a backslash
        $this->wiki->source = str_replace("\\\n", "", $this->wiki->source);

    }

}
