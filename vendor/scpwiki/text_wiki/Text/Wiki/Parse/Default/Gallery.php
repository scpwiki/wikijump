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
 * Creates a simple gallery.
 *
 * @category Text
 *
 * @package Text_Wiki
 *
 * @author Michal Frackowiak
 *
 */

class Text_Wiki_Parse_Gallery extends Text_Wiki_Parse {

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
                        '^' . 
                        '\[\[gallery(\s[^\]]*?)?\]\]' . 
                        '(?:((?:\n:\s[^\n]+)+)\n' . 
                        '\[\[\/gallery\]\])?' . 
                        '/msix';

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

        $attr = $this->getAttrs(trim($matches[1]));
        $sources = $matches[2];
        $options = array();
        if ($attr['size']) {
            $options['size'] = $attr['size'];
        }
        $options['sources'] = $sources;
        $token = $this->wiki->addToken($this->rule, $options);

        return "\n\n" . $token . "\n\n";

    }
}
