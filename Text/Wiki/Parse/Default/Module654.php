<?php

/**
 *
 * Parses for modules.
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
 * Parses for modules.
 *
 * @category Text
 *
 * @package Text_Wiki
 *
 * @author Michal Frackowiak
 *
 */

class Text_Wiki_Parse_Module654 extends Text_Wiki_Parse {

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

    public $regex = "/^\[\[module654\s([a-z0-9_\-\/]+)(\s+.*?)?\]\]\n(?:(.*?)\[\[\/module\]\])?/ims";

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

    function process(&$matches) {

        $moduleName = trim($matches[1]);

        // are there additional attribute arguments?
        $attr = trim($matches[2]);

        $body = trim($matches[3]); // optional module "body" between tags


        $options = array('moduleName' => $moduleName, 'attr' => $attr,
            'module_body' => $body);

        return "\n\n" . $this->wiki->addToken($this->rule, $options) . "\n\n";
    }
}
